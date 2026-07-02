# CODEX Workplan S04

## Scope

Build only the M1/W0 deterministic world harness. Do not connect it to AM core or a perception bridge.

S04 also includes the requested preflight fix for `hebb::add_link_delta` so clamp/saturation cannot apply sub-`eps_log` W changes.

## Files and Order

1. `src/core/hebb.rs`
   - Compute proposed W state before mutating.
   - Apply only when the actual before/after change is at least `eps_log`.

2. `tests/diff_integrity.rs`
   - Add a sub-`eps_log` W growth/saturation guard while keeping existing S03 coverage.

3. `src/world/*`
   - Add deterministic W0 harness modules:
     - `theta.rs`
     - `rng.rs`
     - `action.rs`
     - `grid.rs`
     - `observation.rs`
     - `runner.rs`
     - `script.rs`
   - Keep runtime single-threaded and deterministic.
   - Use seeded SplitMix64-style PRNG; no `rand` crate.
   - Keep observation JSON neutral.

4. `src/lib.rs`
   - Export the `world` module.

5. `src/cli/commands.rs`
   - Add `am world-run`.
   - Parse text scripts into closed `Action` enum.
   - Write observation JSONL and world trace JSONL.
   - Print deterministic inspection dumps when requested.

6. Tests
   - `tests/world_determinism.rs`
   - `tests/observation_schema.rs`
   - `tests/world_action_enum.rs`
   - `tests/world_semantic_quarantine.rs`
   - `tests/world_golden_trace.rs`
   - `tests/world_cli_smoke.rs`

7. Docs
   - `docs/W0_WORLD.md`
   - `docs/OBSERVATION_SCHEMA.md`
   - `docs/BUILD_REPORT_S04.md`

## Verification Commands

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test -- --nocapture`
- `cargo run -- world-run --map-seed 7 --rule-seed 3 --script demo_actions.txt --obs-out data/w0_obs.jsonl --trace-out data/w0_trace.jsonl --dump-every 10`
- Hash obs/trace, rerun, and prove hashes match.

## Risks

- Semantic quarantine can be violated by harmless-looking field names. Observation-facing names must stay neutral.
- Tests must not accidentally require AM core to parse world JSON; S04 must stop at JSONL files.
- `chrono` already exists in Cargo, but deterministic runtime paths must not call wall-clock APIs.

## Deferrals

- No perception bridge.
- No P prototypes.
- No object identity/tracks.
- No relations, schemas, planner, replay, transfer benchmark, or AM core piping.
- No new theta sweep or default theta change.
