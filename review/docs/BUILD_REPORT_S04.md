# AM001 S04 Build Report

## Workstreams

- Preflight W fix: `hebb::add_link_delta` now computes the actual clamped before/after W value first and skips mutation when `abs(after-before) < eps_log`.
- W regression coverage: added `sub_eps_w_growth_and_saturation_are_not_applied_or_logged` to prove sub-eps growth and clamp saturation are neither applied nor logged.
- W0 world harness: added deterministic seeded grid world modules under `src/world`.
- Observation boundary: added neutral observation JSON structs and schema docs. W0 writes JSONL only; it does not call AM core.
- Runner/CLI: added `am world-run` to parse text scripts into the closed `Action` enum and write observation/trace JSONL.
- Quarantine tests: added world determinism, schema, action enum, golden hash, CLI smoke, and semantic/runtime lint tests.

## Commands Run

```text
$ cargo fmt --check
```

Result: exit 0, no stdout.

```text
$ cargo clippy -- -D warnings
    Checking am001 v0.0.1 (/Users/anon5376/Desktop/AM001)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.94s
```

```text
$ cargo test -- --nocapture
test cli_init_step_text_and_dump_smoke ... ok
test partial_cue_completes_linked_assemblies_without_foreign_spill ... ok
test alternating_evidence_opens_and_same_sign_evidence_closes ... ok
test fixed_stream_and_midway_snapshot_are_deterministic ... ok
test sub_eps_w_growth_and_saturation_are_not_applied_or_logged ... ok
test diff_integrity_covers_contradiction_free_reuse_merge_and_sub_eps_w_decay ... ok
test every_changed_scalar_and_link_has_exactly_one_trace_record ... ok
test architecture_drift_scan_rejects_forbidden_dependencies_and_python ... ok
test stale_low_baseline_rows_free_merge_and_protected_rows_survive ... ok
test dump_shows_min_axis_certainty_and_axes_lists_all_axes ... ok
test snapshot_version_mismatch_is_refused_clearly ... ok
test observations_roundtrip_and_use_only_neutral_fields ... ok
test rule_parser_accepts_supported_commands_and_rejects_bad_input ... ok
test repeated_fact_reinforces_one_row_without_duplicates ... ok
test snapshot_roundtrip_is_byte_exact ... ok
stale_rowref_demo: stale_ref=0:1 current_ref=0:3 tick_unchanged=3
test reused_row_rejects_stale_rowref_and_generation_mutations_are_logged ... ok
stale_w_links_demo: freed_row=1 reused_row=1 removed_w_records=2 c_active_after_cue=0.000000
test freed_row_reuse_cannot_inherit_stale_w_edges ... ok
test tiny_sweep_writes_well_formed_csv ... ok
test script_parsing_maps_to_closed_action_enum_and_rejects_unknowns ... ok
test world_run_writes_jsonl_and_deterministic_dump_lines ... ok
test same_seeds_and_script_produce_byte_identical_jsonl ... ok
world_golden_obs_hash=c0129c72e355a41f1b74ae2cadb07c8a82606d454c71011a640aadf417bc6229
world_golden_trace_hash=4f3b634b3ebd56231098d6fd0700a5a9e840b20a286579ff05bd9b46d162c699
test fixed_script_matches_golden_world_hashes ... ok
test world_observations_and_runtime_paths_stay_quarantined ... ok
Doc-tests am001: ok
```

Full result: 24 integration tests passed, 0 failed. Lib/main/doc tests had 0 tests and passed.

AM theta hashes printed by tests:

- Default AM theta: `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`
- Completion theta: `e5d8e36692df55f5809a3dbb9acf43b6999c61957f49a702df70d7454bb197a0`

Default W0 `WorldTheta` hash:

- `85c1e007fbdca12678c44d2f350fd3c42e4e8a747a41b738158ba1882e652d39`

## W0 Demo

```text
$ cargo run -- world-run --map-seed 7 --rule-seed 3 --script demo_actions.txt --obs-out data/w0_obs.jsonl --trace-out data/w0_trace.jsonl --dump-every 10
   Compiling am001 v0.0.1 (/Users/anon5376/Desktop/AM001)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.78s
     Running `target/debug/am world-run --map-seed 7 --rule-seed 3 --script demo_actions.txt --obs-out data/w0_obs.jsonl --trace-out data/w0_trace.jsonl --dump-every 10`
world tick=10 self=(4, 1) energy=6 reward=0 blocked=false visible=8
world-run actions=12 obs=data/w0_obs.jsonl trace=data/w0_trace.jsonl
```

First hashes:

```text
466194ff2b1c1a65890422ccb95aff3ff7556c4fd9e9ad7c68460c3d4295f96f  data/w0_obs.jsonl
e6181e88b5e7a56b595a823f7fc379f1e9f9e50dd0251bd2822586111a971ee6  data/w0_trace.jsonl
```

Rerun:

```text
$ cargo run -- world-run --map-seed 7 --rule-seed 3 --script demo_actions.txt --obs-out data/w0_obs.jsonl --trace-out data/w0_trace.jsonl --dump-every 10
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/am world-run --map-seed 7 --rule-seed 3 --script demo_actions.txt --obs-out data/w0_obs.jsonl --trace-out data/w0_trace.jsonl --dump-every 10`
world tick=10 self=(4, 1) energy=6 reward=0 blocked=false visible=8
world-run actions=12 obs=data/w0_obs.jsonl trace=data/w0_trace.jsonl
```

Second hashes:

```text
466194ff2b1c1a65890422ccb95aff3ff7556c4fd9e9ad7c68460c3d4295f96f  data/w0_obs.jsonl
e6181e88b5e7a56b595a823f7fc379f1e9f9e50dd0251bd2822586111a971ee6  data/w0_trace.jsonl
```

Hashes match.

Observation sample:

```json
{"tick":1,"map_seed":7,"rule_seed":3,"self":{"x":3,"y":1},"energy_bucket":5,"reward_delta":0,"blocked":false,"visible_entities":[{"local_id":1,"shape_id":1,"color_id":101,"size":1,"x":2,"y":0,"dx":-1,"dy":-1,"adjacent":false,"distance_to_self":2},{"local_id":2,"shape_id":2,"color_id":102,"size":2,"x":3,"y":3,"dx":0,"dy":2,"adjacent":false,"distance_to_self":2}]}
```

Trace sample:

```json
{"tick":1,"map_seed":7,"rule_seed":3,"theta_hash":"85c1e007fbdca12678c44d2f350fd3c42e4e8a747a41b738158ba1882e652d39","action":"N","x":3,"y":1,"energy_bucket":5,"reward_delta":0,"blocked":false,"contact_ids":[],"rule_codes":[]}
```

## Tests Added

- `tests/world_determinism.rs`: same seeds/script produce byte-identical observation and trace JSONL; different map seed changes hashes.
- `tests/observation_schema.rs`: observations serialize/deserialize and expose only neutral schema fields.
- `tests/world_action_enum.rs`: script parsing maps to the closed action enum and rejects unknown actions.
- `tests/world_semantic_quarantine.rs`: observation JSON and non-world core/perception/learner/planner paths reject semantic blacklist words; deterministic runtime paths reject LLM/client/thread/time/rand tokens.
- `tests/world_golden_trace.rs`: fixed W0 script has stable observation and trace hashes.
- `tests/world_cli_smoke.rs`: `world-run` writes obs/trace JSONL and prints deterministic dump lines.

## Files Changed

- Added `src/world/mod.rs`
- Added `src/world/theta.rs`
- Added `src/world/rng.rs`
- Added `src/world/action.rs`
- Added `src/world/grid.rs`
- Added `src/world/observation.rs`
- Added `src/world/runner.rs`
- Added `src/world/script.rs`
- Added `docs/W0_WORLD.md`
- Added `docs/OBSERVATION_SCHEMA.md`
- Added `docs/CODEX_WORKPLAN_S04.md`
- Added `docs/BUILD_REPORT_S04.md`
- Added `demo_actions.txt`
- Added world tests listed above.
- Changed `src/cli/commands.rs`
- Changed `src/core/hebb.rs`
- Changed `src/lib.rs`
- Changed `tests/diff_integrity.rs`
- Changed `.gitignore` to ignore generated `data/*.jsonl`.

## Deviations / Decisions

- No perception bridge was built.
- No AM core call reads observation JSON. `world-run` stops at JSONL.
- No object identity/tracks, concepts, relations, schemas, planner, replay, or transfer benchmark were built.
- W0 trace serializes the closed `Action` enum as JSON strings for audit readability. Runtime still uses the enum; script text exists only at CLI/script parsing.
- World internals use neutral numeric `rule_code`s instead of semantic role names. This is stricter than the allowance that world internals may contain semantics.
- No snapshot format change was made because AM core state layout did not change.

## Known Limitations

- W0 is intentionally small and fully observable.
- Entity appearances are unique by numeric ids, not by rendered assets.
- Hidden rules are simple numeric response codes; there is no perception bridge to learn them yet.
- The runner is library/CLI only. There is no long-lived runtime loop.
- `chrono` remains an existing dependency, but S04 deterministic runtime paths do not call wall-clock APIs.

## Exact Next Tasks

1. S05 / M2 perception bridge: observation JSON -> neutral percepts F=12 -> validated AM001 Events.
2. Add golden fixture files only if future reviewers need file-based fixtures instead of hash-based tests.
3. Decide whether W0 trace should use numeric action ids in addition to enum serialization before any non-human consumer reads traces.
4. Keep semantic-quarantine lint active before adding perception or learner modules.
