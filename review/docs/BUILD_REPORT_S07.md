# AM001 S07 Build Report

## Workstreams

- Added P03 percept bridge:
  - `docs/PERCEPT_SCHEMA.md`
  - `docs/PERCEPT_AXIS_MAP.md`
  - `src/percept/mod.rs`
  - `am pipe`
- Implemented appearance feature vector `F=12`:
  - 5 shape slots
  - 6 color slots
  - 1 size scalar
  - pose remains separate state data and is not in the vector
- Per observation, bridge emits validated AM `Event`s:
  - `cue loc_<local_id> strength=0.8`
  - entity asserts: `temporal_near`, `concreteness`, `novelty`
  - adjacent links only: `trk_0 loc_<local_id> 0.6`
  - self row asserts: `temporal_near`, `value`, `risk`, and `constraint_relevance` when blocked
- Temporary novelty uses a session-local feature signature set because P is not present yet.
- Fixed render glyph stability:
  - glyph table is built once per episode
  - later shapes append monotonically
  - frame header now says `held_shape=`
- Moved `Theta::default().eta_w` from `0.15` to `0.30`.
- Added A17 to `docs/AM_CORE_V0_AMENDMENTS.md`.
- Added `SweepGrid.measure_margin`:
  - default `true`
  - `sweep/grid.json` sets `false` for search grids
  - skipped margin is an empty CSV cell, never a fabricated value

## Commands Run

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.92s
...
test result: ok. 42 passed; 0 failed
```

```text
$ cargo run --quiet -- sweep --grid /tmp/am_s07_full_confirm.json --out sweep/results_s07_full_confirm.csv
wrote sweep/results_s07_full_confirm.csv total_points=1 invalid_points=0 evaluated_points=1 all_pass_points=1
```

```text
$ cargo run --quiet -- pipe --map-seed 7 --rule-seed 3 --script demo_actions.txt --dump-every 10
pipe tick=10 core_tick=10 emitted_events=10
...
pipe actions=12 observations=12 events=12 core_tick=12 termination=ScriptEnd
```

```text
$ cargo run --quiet -- world-run --map-seed 7 --rule-seed 3 --script /tmp/am_w0_demo_try.txt --obs-out /tmp/am_s07_render_obs.jsonl --trace-out /tmp/am_s07_render_trace.jsonl --render
world-run actions=12 ran=12 termination=Exit obs=/tmp/am_s07_render_obs.jsonl trace=/tmp/am_s07_render_trace.jsonl
```

## Default Theta

New default core theta hash:

```text
6905c5f8beede6bd3dfd9fc403626eafb433488291667610d11f0db54f23520c
```

Default world theta hash:

```text
bc40a491d984be4d88644958dd25d7270f1db0f57a8fe0632214c82480b75bf6
```

Custom theta hashes used by non-default tests:

```text
diff_sub_eps_growth=52e0d0982b45331f7d050da0a8479e9428addfbbee66943037bde5b114afc7d6
diff_free_reuse_stale_w=144bdf9798e23656f9446c3402c98636a05aa55f7ee7801ee3d472592d660a4a
resting_field=a0ba109b40b5887b1561f856f1b2ec2073294630348fb6fea1ce8bb2e1bacd93
link_decay_alive=7c051233480a8d550a4cd577653dd685aba4260fd32869a0c6df651dd65fd3ae
```

## Full-Scale Confirmation

Committed file:

```text
sweep/results_s07_full_confirm.csv
```

Full row:

```text
6905c5f8beede6bd3dfd9fc403626eafb433488291667610d11f0db54f23520c,4096,30,true,0.55,1.4,1.2,3,3,0.3,0,0.4,0.01,0.6,0.05,0.25,20,true,true,true,true,true,true,true,1.000000,0.000000,1.000000,""
```

This confirms A17:

```text
completion_recall=1.000000
completion_contamination=0.000000
recall_margin_095=1.000000
```

## R1 Evidence

Test:

```text
cli::render::tests::barrier_letter_stays_stable_after_removal
```

Before removal at tick 7:

```text
tick=7 energy=13 delta=0 held_shape=8 blocked=false failed=false termination=None
..#......
...#.#...
..#...#..
...#.....
~oA@.....
.o~#X....
a....B.#.
```

After removal at tick 8:

```text
tick=8 energy=13 delta=0 held_shape=8 blocked=false failed=false termination=None
..#......
...#.#...
..#...#..
...#.....
~o.@.....
.o~#X....
a....B.#.
```

The removed barrier was `A`; the surviving barrier remains `B`.

## Pipe Demo

Command:

```text
cargo run --quiet -- pipe --map-seed 7 --rule-seed 3 --script demo_actions.txt --dump-every 10
```

Output:

```text
pipe tick=10 core_tick=10 emitted_events=10
loc_1        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
loc_2        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
loc_3        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
loc_4        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
loc_5        [temporal_near +0.79  concreteness +0.79]  a=0.70 b=0.12 cert=0.84 age=0
loc_6        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
loc_7        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
loc_8        [temporal_near +0.79  concreteness +0.79  novelty +0.02]  a=0.70 b=0.12 cert=0.84 age=0
pipe actions=12 observations=12 events=12 core_tick=12 termination=ScriptEnd
```

This demonstrates rows appearing from pure observation events. Labels are temporary `loc_<local_id>` labels and must be replaced by S04 `trk_<n>` labels when the tracker store lands.

## Test Results

- `cli::render::tests::barrier_letter_stays_stable_after_removal`: pass.
- `world::grid::tests::wall_blocks_and_does_not_change_energy`: pass.
- `world::grid::tests::barrier_blocks_pre_open_matching_portable_removes_it_and_cell_becomes_passable`: pass.
- `world::grid::tests::wrong_class_and_empty_hands_open_fail`: pass.
- `world::grid::tests::pickup_own_cell_and_holding_slot_failure`: pass.
- `world::grid::tests::drop_success_and_occupied_cell_failure`: pass.
- `world::grid::tests::consumable_arithmetic_and_removal`: pass.
- `world::grid::tests::hazard_applies_per_occupied_tick_including_wait`: pass.
- `world::grid::tests::step_cost_fires_exactly_on_interval_ticks`: pass.
- `world::grid::tests::energy_death_exit_and_tick_cap_terminate`: pass.
- `world::grid::tests::rule_seed_drives_table_and_shape_permutation_map_seed_drives_positions`: pass.
- `cli_init_step_text_and_dump_smoke`: pass, default core theta.
- `partial_cue_completes_linked_assemblies_without_foreign_spill`: pass, default core theta.
- `alternating_evidence_opens_and_same_sign_evidence_closes`: pass, default core theta.
- `fixed_stream_and_midway_snapshot_are_deterministic`: pass, default core theta.
- `sub_eps_w_growth_and_saturation_are_not_applied_or_logged`: pass, custom theta `52e0d098...`.
- `every_changed_scalar_and_link_has_exactly_one_trace_record`: pass, default core theta.
- `diff_integrity_covers_contradiction_free_reuse_merge_and_sub_eps_w_decay`: pass, includes custom theta `144bdf97...`.
- `architecture_drift_scan_rejects_forbidden_dependencies_and_python`: pass.
- `stale_low_baseline_rows_free_merge_and_protected_rows_survive`: pass, default core theta.
- `snapshot_version_mismatch_is_refused_clearly`: pass, default core theta.
- `snapshot_theta_violating_a15c_is_refused_clearly`: pass, default core theta snapshot mutated invalid.
- `dump_shows_min_axis_certainty_and_axes_lists_all_axes`: pass, default core theta.
- `nonzero_link_decay_decreases_and_prunes_with_cause_records`: pass, custom theta `7c051233...`.
- `observations_roundtrip_and_use_only_neutral_fields`: pass, default W0 theta.
- `rule_parser_accepts_supported_commands_and_rejects_bad_input`: pass.
- `same_observation_produces_identical_events_from_same_initial_bridge_state`: pass.
- `percept_labels_are_opaque_and_events_do_not_leak_world_words`: pass.
- `world_percept_core_double_run_is_byte_deterministic`: pass.
- `percept_events_pass_validator_and_use_existing_axes`: pass.
- `repeated_fact_reinforces_one_row_without_duplicates`: pass, default core theta.
- `empty_ticks_drive_allocated_field_quiet_under_a15c`: pass, custom theta `a0ba109b...`.
- `snapshot_roundtrip_is_byte_exact`: pass, default core theta.
- `reused_row_rejects_stale_rowref_and_generation_mutations_are_logged`: pass, default core theta.
- `freed_row_reuse_cannot_inherit_stale_w_edges`: pass, custom theta `144bdf97...`.
- `tiny_sweep_writes_well_formed_csv`: pass, confirms empty `recall_margin_095` when `measure_margin=false`.
- `script_parsing_maps_to_closed_action_enum_and_rejects_unknowns`: pass.
- `world_run_writes_jsonl_and_deterministic_dump_lines`: pass.
- `same_seeds_and_script_produce_byte_identical_jsonl`: pass.
- `fixed_script_matches_golden_world_hashes`: pass.
- `world_observations_and_runtime_paths_stay_quarantined`: pass.
- `reserved_world_tier_flags_are_rejected`: pass.

## Measure Margin Policy

`measure_margin=false` was used only for search-grid behavior:

```text
sweep/grid.json
tests/sweep_smoke.rs
```

The load-bearing full-scale confirmation used `measure_margin=true`.

## Files Changed By S07

```text
docs/AM_CORE_V0_AMENDMENTS.md
docs/BUILD_REPORT_S07.md
docs/CODEX_WORKPLAN_S07.md
docs/PERCEPT_AXIS_MAP.md
docs/PERCEPT_SCHEMA.md
src/cli/commands.rs
src/cli/render.rs
src/core/theta.rs
src/eval/criteria.rs
src/eval/sweep.rs
src/lib.rs
src/percept/mod.rs
sweep/grid.json
sweep/results_s07_full_confirm.csv
tests/percept_determinism.rs
tests/percept_no_semantics.rs
tests/percept_pipe_determinism.rs
tests/percept_validity.rs
tests/sweep_smoke.rs
```

Pre-existing local edits were observed and preserved but are not S07 work:

```text
README.md
dashboard/
src/cli/dashboard.rs
src/cli/mod.rs dashboard export
src/cli/commands.rs dashboard command
```

## Deviations

- P store is not present, so novelty uses a deterministic session-local feature signature set as specified.
- Temporary `loc_<local_id>` labels are used and documented as replaced-by-S04.
- The self-row energy mapping assumes W0's current max bucket of 20. If that becomes dynamic at the bridge boundary, this should move into a theta-style percept config.

## Known Limitations

- Perception emits no causal assertions, schemas, or planning facts.
- Pose is available in `Percept` but not written into appearance features.
- `am pipe` is an end-to-end demo path, not a persistent pipeline service.
- Existing dashboard work in the tree is not audited by this report.

## Exact Next Tasks

1. Replace temporary `loc_<local_id>` labels with S04 track labels.
2. Add the P store and replace session-local novelty with prototype cosine novelty.
3. Build schema learning over the typed action/effect vocabulary.
4. Add report-level artifacts for comparing learned schemas across rule seeds.
