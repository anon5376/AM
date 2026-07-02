# W0 World Harness

W0 is a deterministic, fully observable grid harness. It does not call AM core and does not build a perception bridge.

## Determinism

Inputs:

- `WorldTheta`
- `map_seed`
- `rule_seed`
- closed `Action` script

Outputs:

- ordered observation JSONL
- ordered world trace JSONL

The same inputs produce byte-identical JSONL outputs. World initialization uses an internal SplitMix64-style seeded PRNG implemented in Rust; no `rand` crate is used.

## Runtime Boundary

W0 stops at JSONL. The `world-run` CLI command writes observations and audit trace files, but does not feed AM core, does not parse observations into AM events, and does not call any LLM, database, RAG, thread loop, or network client.

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

The map is a fixed-size grid from `WorldTheta`. W0 has full observability and static unique-appearance entities. Entity ordering is stable by `local_id`.

The hidden rule table is owned by the world engine and is initialized from `rule_seed`. Observation-facing data exposes only neutral numeric identifiers and geometry.

## CLI

```bash
cargo run -- world-run \
  --map-seed 7 \
  --rule-seed 3 \
  --script demo_actions.txt \
  --obs-out data/w0_obs.jsonl \
  --trace-out data/w0_trace.jsonl \
  --dump-every 10
```

`--dump-every` prints deterministic inspection lines for humans. These lines are not read by the runtime.
