# CODEX Workplan S05

## Scope

Make the AM attractor field discriminative at `t=20` by implementing A15 sustained input and the resting-field invariant, then re-sweep for one full-scale default theta passing all core criteria.

## Files and Order

1. `docs/AM_CORE_V0_AMENDMENTS.md`
   - Append A15 verbatim.

2. `src/core/theta.rs`
   - Add `k_i: f32`.
   - Validate A15c resting-field invariant.
   - Update `Theta::default()` after the sweep identifies a full-scale all-pass point.

3. `src/core/resolve.rs`, `src/core/settle.rs`, `src/core/step.rs`
   - Build step-local external input from cues/asserts.
   - Hold it across all settle iterations.
   - Clear it after the step by construction.
   - Do not change logging shape.

4. `src/eval/criteria.rs`, `src/eval/sweep.rs`, `sweep/grid.json`
   - Add scaled sweep config: 512 rows, 12 completion assemblies.
   - Extend grid with `th0`, `k_i`, `beta`, `lam_settle`, `gamma`, `eta_w`, `del_w`.
   - Skip/count invalid theta points.
   - Confirm candidate at full scale before changing default.

5. Tests
   - Add `tests/resting_field.rs`.
   - Update tests and theta JSON for new `k_i` field.
   - Delete `tests/theta/completion.json` only after default passes all full-scale criteria.
   - Regenerate any golden hashes that legitimately change.

6. `docs/BUILD_REPORT_S05.md`
   - Capture commands, sweep runtime/counts, candidate selection, theta hashes, deleted override, golden churn, deviations, limitations, and next tasks.

## Verification Commands

- `cargo fmt && cargo clippy && cargo test`
- `cargo run --release -- sweep --grid sweep/grid.json --out sweep/results_s05_a15a.csv`
- Full-scale confirmation for the winning theta.
- Standard demo transcript.
- Dump after 10 empty ticks proving the field goes quiet.

## Risks

- A15c invalidates many old theta files; tests must be updated instead of weakening invariant.
- Scaled sweep can find a false positive; full-scale confirmation is mandatory before changing default.
- Golden world hashes should not change because S05 touches AM dynamics, not W0.

## Escalation

1. If A15a yields zero all-pass points, implement A15b and re-sweep.
2. If A15b still yields zero all-pass points, document A15d and apply k-WTA mask every 5 settle iterations.

## Deferrals

- No runtime LLM, RAG, database, Python, or thread loop.
- No perception bridge, planner, schemas, relation stores, tracks, or replay.
- No default with `t < 20`.
- No weakening completion thresholds.
