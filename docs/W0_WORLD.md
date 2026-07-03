# W0 World Harness

W0 is a deterministic, fully observable grid harness. It does not call AM core and does not build a perception bridge.

## Runtime Boundary

W0 stops at JSONL. The `world-run` CLI command writes observations and audit trace files, but does not feed AM core, does not parse observations into AM events, and does not call any LLM, database, RAG, thread loop, or network client.

## Determinism

Inputs:

- `WorldTheta`
- `map_seed`
- `rule_seed`
- closed `Action` script

Outputs:

- ordered observation JSONL
- ordered world trace JSONL
- optional CLI-only ASCII frames

The same inputs produce byte-identical JSONL outputs. World initialization uses the local SplitMix64 harness; no `rand` crate is used.

`map_seed` drives placement and cosmetic per-entity variation. `rule_seed` drives the hidden class-to-shape permutation and the hidden portable-class to barrier-class matching table.

## Actions

Runtime actions use the closed Rust enum:

```text
N, S, E, W, PickUp, Drop, Open, Wait
```

Tie-break order is fixed as:

```text
N, S, E, W, PickUp, Drop, Open, Wait
```

Scripts may use those exact tokens. Unknown tokens are rejected.

## World Shape

Default `WorldTheta`:

- size: `9x7`
- classes: 2 portable classes, 2 barrier classes
- mix: 8 walls, 1 item per portable class, 1 barrier per barrier class, 3 consumables, 2 hazards, 1 exit
- energy: start 10, max 20, min 0
- effects: consumable +3, hazard -2, step cost -1 every 20 ticks
- `step_limit=400`

The hidden behavior class is never an observation field. Appearance carries class structure through `shape_id`: all entities of one behavior class share one shape, and distinct behavior classes have distinct shapes under the `rule_seed` permutation. `color_id` and `size` are cosmetic.

No W0 solvability guarantee exists. Scripted demos and tests pin seed pairs known to exercise the intended mechanics.

## Mechanics

Movement into walls and present barriers is blocked. A blocked move leaves position unchanged, sets `blocked=true`, and has zero energy delta.

The agent has one holding slot. `PickUp` acts on the current cell and succeeds only when standing on a portable entity with empty hands. The entity leaves the grid while held. `Drop` succeeds only when the current cell is empty. `Open` targets adjacent barriers in `N,S,E,W` order and succeeds only when the held portable class maps to that barrier class. Successful open removes the barrier; the held entity is not consumed.

Consumables are removed automatically when a successful move ends on their cell and add energy. Hazards persist and subtract energy on every occupied tick, including `Wait`. Step cost applies on ticks divisible by `step_cost_interval`.

Termination causes are a closed enum in trace output and `EpisodeOutput`: `Exit`, `EnergyDeath`, `TickCap`, `ScriptEnd`. The runner stops mid-script on the first non-script termination and stamps the final trace line with the cause.

Reserved tier flags `twins`, `motion`, `confound`, `vision_radius`, and `rule_resample` intentionally return `UnimplementedTier` in W0.

## CLI

```bash
cargo run -- world-run \
  --map-seed 7 \
  --rule-seed 3 \
  --script demo_actions.txt \
  --obs-out data/w0_obs.jsonl \
  --trace-out data/w0_trace.jsonl \
  --dump-every 10 \
  --render
```

`--dump-every` prints deterministic inspection lines for humans. `--render` prints CLI-only ASCII frames. Neither channel is read by runtime learning code.
