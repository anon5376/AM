# S07 Workplan

## Mission

Unfreeze P03 by adding a deterministic observation-to-event percept bridge, fixing CLI render glyph stability, moving default `eta_w` to `0.30`, and making completion margin optional for search sweeps while mandatory for full-scale confirmation.

## Files And Order

1. Percept bridge:
   - Add `docs/PERCEPT_SCHEMA.md` and `docs/PERCEPT_AXIS_MAP.md`.
   - Add `src/percept/` with feature extraction, session-local novelty, neutral label generation, and event emission through `Event::validate_and_clamp`.
   - Export `percept` from `src/lib.rs`.
   - Add `am pipe` in `src/cli/commands.rs` while preserving the existing dashboard command edits.

2. Render rider R1:
   - Refactor `src/cli/render.rs` to use an episode-stable glyph map and rename frame header `held=` to `held_shape=`.
   - Update world-run rendering to build the glyph map once per episode.
   - Add a render unit test for barrier-letter stability after removal.

3. Theta and sweep riders R2/R3:
   - Change `Theta::default().eta_w` from `0.15` to `0.30`.
   - Append A17 to `docs/AM_CORE_V0_AMENDMENTS.md`.
   - Add `SweepGrid.measure_margin`, default true; set `sweep/grid.json` false for search grids.
   - Keep full-scale one-point confirmation with margin true and commit the CSV.

4. Tests:
   - Add percept determinism, validity, no-semantics, and end-to-end world-to-percept-to-core determinism tests.
   - Update any tests mechanically affected by the default theta hash.
   - Run `cargo fmt && cargo clippy -- -D warnings && cargo test`.

5. Report:
   - Run `am pipe --map-seed 7 --rule-seed 3 --script demo_actions.txt --dump-every 10`.
   - Write `docs/BUILD_REPORT_S07.md` with commands, theta hashes, full-scale confirm row, render stability evidence, files changed, deviations, limitations, and exact next tasks.

## Risks

- `src/percept` is scanned by semantic quarantine, so implementation names must stay neutral and avoid world-mechanic tokens.
- Existing uncommitted dashboard edits touch CLI files; S07 must preserve them.
- Default `eta_w=0.30` changes the default theta hash and may alter test dynamics; no thresholds may be weakened.
- `measure_margin=false` must never fabricate a margin value.

## Deferrals

- No schema learner.
- No planner.
- No causal assertions from perception.
- Temporary `loc_<local_id>` labels remain documented as replaced-by-S04 track labels.
