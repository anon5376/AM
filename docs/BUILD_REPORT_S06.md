# AM001 S06 Build Report

## Workstreams

- Replaced W0 `rule_code` reward pings with a hidden behavior-class system in `src/world/classes.rs`.
- Refactored W0 mechanics:
  - seeded class-to-shape permutation from `rule_seed`
  - seeded portable-to-barrier matching table from `rule_seed`
  - map placement and cosmetic variation from `map_seed`
  - blocking walls and present barriers
  - held slot, pickup, drop, open, removal, consumable, hazard, step cost, and termination causes
- Added neutral observation field `held_shape_id`; behavior classes, passability, open-state, and matching tables remain absent from observation JSON.
- Added CLI-only `--render` ASCII frames.
- Added reserved tier flag validation for `twins`, `motion`, `confound`, `vision_radius`, and `rule_resample`.
- Landed S05 audit riders C1-C3:
  - `AmState::new` validates `Theta`
  - snapshot load validates embedded `Theta` after v4 format checks
  - A16 appended
  - link decay aliveness test added
  - completion criterion and sweep CSV now report `recall_margin_095`

## Commands Run

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
...
test result: ok. 37 passed; 0 failed
```

```text
$ cargo run --quiet -- sweep --grid /tmp/am_s06_default_confirm.json --out /tmp/am_s06_default_confirm.csv
wrote /tmp/am_s06_default_confirm.csv total_points=1 invalid_points=0 evaluated_points=1 all_pass_points=1
```

```text
$ cargo run --quiet -- bench-step --snapshot /tmp/am_s06_bench.bin --events 1000
bench-step events=1000 mean_us=359 max_us=561
```

An exploratory full `sweep/grid.json` run was stopped after it became clear C3 margin instrumentation makes the 1,944-point grid materially slower. It was not used as verification; the required committed-theta margin was captured with the one-point full-scale sweep above.

## Test Results And Theta Hashes

Default core theta hash:

```text
dce727473be3778453afda3a214d6220ff8bca191a8ec731d151e9b301af3952
```

Default W0 `WorldTheta` hash:

```text
bc40a491d984be4d88644958dd25d7270f1db0f57a8fe0632214c82480b75bf6
```

Custom theta hashes used by non-default tests:

```text
diff_sub_eps_growth=52e0d0982b45331f7d050da0a8479e9428addfbbee66943037bde5b114afc7d6
diff_free_reuse_stale_w=448e3e49829607c9f7b271de60f4ec74e3016b4e80f642547382b328e7e77cad
resting_field=a0ba109b40b5887b1561f856f1b2ec2073294630348fb6fea1ce8bb2e1bacd93
link_decay_alive=7c051233480a8d550a4cd577653dd685aba4260fd32869a0c6df651dd65fd3ae
```

Results:

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
- `diff_integrity_covers_contradiction_free_reuse_merge_and_sub_eps_w_decay`: pass, includes custom `448e3e49...`.
- `architecture_drift_scan_rejects_forbidden_dependencies_and_python`: pass.
- `stale_low_baseline_rows_free_merge_and_protected_rows_survive`: pass, default core theta.
- `snapshot_version_mismatch_is_refused_clearly`: pass, default core theta.
- `snapshot_theta_violating_a15c_is_refused_clearly`: pass, default core theta snapshot mutated to invalid embedded theta for the negative check.
- `dump_shows_min_axis_certainty_and_axes_lists_all_axes`: pass, default core theta.
- `nonzero_link_decay_decreases_and_prunes_with_cause_records`: pass, custom theta `7c051233...`.
- `observations_roundtrip_and_use_only_neutral_fields`: pass, default W0 theta.
- `rule_parser_accepts_supported_commands_and_rejects_bad_input`: pass.
- `repeated_fact_reinforces_one_row_without_duplicates`: pass, default core theta.
- `empty_ticks_drive_allocated_field_quiet_under_a15c`: pass, custom theta `a0ba109b...`.
- `snapshot_roundtrip_is_byte_exact`: pass, default core theta.
- `reused_row_rejects_stale_rowref_and_generation_mutations_are_logged`: pass, default core theta.
- `freed_row_reuse_cannot_inherit_stale_w_edges`: pass, custom theta `448e3e49...`.
- `tiny_sweep_writes_well_formed_csv`: pass.
- `script_parsing_maps_to_closed_action_enum_and_rejects_unknowns`: pass.
- `world_run_writes_jsonl_and_deterministic_dump_lines`: pass, default W0 theta.
- `same_seeds_and_script_produce_byte_identical_jsonl`: pass, default W0 theta.
- `fixed_script_matches_golden_world_hashes`: pass, default W0 theta.
- `world_observations_and_runtime_paths_stay_quarantined`: pass, default W0 theta.
- `reserved_world_tier_flags_are_rejected`: pass, default W0 theta mutated one flag at a time.

## C1-C3 Evidence

C1 negative test:

```text
snapshot_theta_violating_a15c_is_refused_clearly
```

A16 is present in `docs/AM_CORE_V0_AMENDMENTS.md`.

Default-theta full-scale completion margin:

```text
theta_hash=dce727473be3778453afda3a214d6220ff8bca191a8ec731d151e9b301af3952
completion_recall=1.000000
completion_contamination=0.000000
recall_margin_095=0.133333
```

Core-scope diff list for C1-C3:

```text
docs/AM_CORE_V0_AMENDMENTS.md
src/core/snapshot.rs
src/core/state.rs
src/eval/criteria.rs
src/eval/sweep.rs
tests/inspect_snapshot.rs
tests/link_decay_alive.rs
```

The remaining core-test file edits are mechanical `AmState::new(...).unwrap()` updates after the constructor became fallible.

## Golden Regeneration

Regenerated `tests/world_golden_trace.rs`.

New hashes:

```text
world_golden_obs_hash=6998ec720a46695ef15521e2b0effe24e7da817f8d3de0ba459682af4b0d2be2
world_golden_trace_hash=83790d725e130218eaffa82b2421109aeed3200a424992119187d624b061effc
```

Cause:

- W0 observations gained neutral `held_shape_id`.
- W0 traces gained `action_failed`, `held_id`, `removed_ids`, and `termination`.
- W0 mechanics changed from stateless rule pings to class-shaped entities, held state, removal, energy effects, and termination.

## Demo

Pinned seed pair:

```text
map_seed=7
rule_seed=3
script=N W PickUp E S S W Open W E E S
```

Command:

```text
$ cargo run --quiet -- world-run --map-seed 7 --rule-seed 3 --script /tmp/am_w0_demo_try.txt --obs-out /tmp/am_w0_demo_obs.jsonl --trace-out /tmp/am_w0_demo_trace.jsonl --render
```

ASCII episode:

```text
tick=1 energy=13 delta=3 held=None blocked=false failed=false termination=None
..#......
...#.#...
..#b@.#..
...#.....
~oA......
.o~#X....
a....B.#.
tick=2 energy=13 delta=0 held=None blocked=false failed=false termination=None
..#......
...#.#...
..#@..#..
...#.....
~oA......
.o~#X....
a....B.#.
tick=3 energy=13 delta=0 held=8 blocked=false failed=false termination=None
..#......
...#.#...
..#@..#..
...#.....
~oA......
.o~#X....
a....B.#.
tick=8 energy=13 delta=0 held=8 blocked=false failed=false termination=None
..#......
...#.#...
..#...#..
...#.....
~o.@.....
.o~#X....
a....A.#.
tick=12 energy=13 delta=0 held=8 blocked=false failed=false termination=Exit
..#......
...#.#...
..#...#..
...#.....
~o.......
.o~#@....
a....A.#.
world-run actions=12 ran=12 termination=Exit obs=/tmp/am_w0_demo_obs.jsonl trace=/tmp/am_w0_demo_trace.jsonl
```

Energy ledger:

```text
tick action  energy delta event
1    N       13     +3    moved onto consumable; entity removed
2    W       13      0    moved onto portable
3    PickUp  13      0    held_id=9
4    E       13      0
5    S       13      0
6    S       13      0
7    W       13      0    adjacent to barrier
8    Open    13      0    removed_id=12; held item retained
9    W       13      0    entered cleared cell
10   E       13      0
11   E       13      0
12   S       13      0    termination=Exit
```

## Files Changed

```text
docs/AM_CORE_V0_AMENDMENTS.md
docs/BUILD_REPORT_S06.md
docs/CODEX_WORKPLAN_S06.md
docs/OBSERVATION_SCHEMA.md
docs/W0_WORLD.md
src/cli/commands.rs
src/cli/mod.rs
src/cli/render.rs
src/core/snapshot.rs
src/core/state.rs
src/eval/criteria.rs
src/eval/sweep.rs
src/world/classes.rs
src/world/grid.rs
src/world/mod.rs
src/world/observation.rs
src/world/runner.rs
src/world/theta.rs
tests/completion.rs
tests/contradiction.rs
tests/determinism.rs
tests/diff_integrity.rs
tests/drift_check.rs
tests/forgetting.rs
tests/inspect_snapshot.rs
tests/link_decay_alive.rs
tests/observation_schema.rs
tests/reinforcement.rs
tests/resting_field.rs
tests/snapshot_roundtrip.rs
tests/stale_rowref.rs
tests/stale_w_links.rs
tests/world_golden_trace.rs
tests/world_tier_flags.rs
```

`AM-0.0.1.zip` remains an unrelated untracked archive and was not touched.

## Deviations

- No perception bridge or core-world wiring was added.
- No solvability search machinery was added; W0 remains seed/script pinned as specified.
- `AmState::new` is now fallible. Tests that construct core state directly unwrap validated theta at the test boundary.
- The full grid sweep was not re-run as a requirement for S06; only the committed default theta was checked for the new C3 margin metric.

## Known Limitations

- W0 has no solvability guarantee.
- ASCII render is an inspection-only CLI surface.
- Behavior class visibility is intentionally only inferable through shape regularity and outcomes in observation JSON.
- Schema learning, perception parsing, and planning remain future work.

## Exact Next Tasks

1. Build the observation-to-event perception bridge as a separate, quarantined layer.
2. Add the schema learner against W0 JSONL traces.
3. Add transfer tests that prove class-shaped appearance regularity supports learning across `rule_seed` changes.
4. Decide whether C3 margin overhead needs a faster implementation before any future large sweep.
