# AM001 S05 Build Report

## Workstreams

- Appended A15 to `docs/AM_CORE_V0_AMENDMENTS.md`.
- Implemented A15a sustained per-step external input in P1/P2:
  - cues contribute `k_i * cue_strength`
  - asserts contribute `k_i * a_init`
  - input is held through all settle iterations and then discarded.
- Added `Theta.k_i` and A15c validation:
  - `beta * sigmoid(-th0) / (1 - lam_settle) < th_act`
- Bumped snapshot format to v4 because serialized `Theta` changed.
- Added scaled sweep support with `SweepScale { n, completion_assemblies }`.
- Replaced the old 729-point grid with the A15a grid and 512-row/12-assembly scale.
- Found and committed one full-scale all-pass default theta at `t=20`.
- Deleted `tests/theta/completion.json`; completion now passes on `Theta::default()`.
- Added `tests/resting_field.rs`.

## Committed Default Theta

Default theta hash:

```text
dce727473be3778453afda3a214d6220ff8bca191a8ec731d151e9b301af3952
```

Changed default values:

```text
lam_settle=0.55
beta=1.4
gamma=1.2
th0=3.0
k_i=3.0
eta_w=0.15
del_w=0.0
t=20
```

A15c resting bound:

```text
beta * sigmoid(-th0) / (1 - lam_settle) = ~0.1476 < th_act 0.25
```

## Commands Run

```text
$ cargo fmt && cargo clippy && cargo test
    Checking am001 v0.0.1 (/Users/anon5376/Desktop/AM001)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.29s
...
test result: ok. 0 failed
```

Full test run result:

- lib tests: 0 passed, 0 failed
- main tests: 0 passed, 0 failed
- doc tests: 0 passed, 0 failed
- integration tests: 25 passed, 0 failed

## Test Results And Theta Hashes

- `cli_init_step_text_and_dump_smoke`: pass.
- `partial_cue_completes_linked_assemblies_without_foreign_spill`: pass, default theta `dce727473be3778453afda3a214d6220ff8bca191a8ec731d151e9b301af3952`.
- `alternating_evidence_opens_and_same_sign_evidence_closes`: pass, default theta.
- `fixed_stream_and_midway_snapshot_are_deterministic`: pass, default theta.
- `sub_eps_w_growth_and_saturation_are_not_applied_or_logged`: pass, default theta.
- `diff_integrity_covers_contradiction_free_reuse_merge_and_sub_eps_w_decay`: pass, default theta.
- `every_changed_scalar_and_link_has_exactly_one_trace_record`: pass, default theta.
- `architecture_drift_scan_rejects_forbidden_dependencies_and_python`: pass.
- `stale_low_baseline_rows_free_merge_and_protected_rows_survive`: pass, default theta.
- `dump_shows_min_axis_certainty_and_axes_lists_all_axes`: pass, default theta.
- `snapshot_version_mismatch_is_refused_clearly`: pass, default theta.
- `observations_roundtrip_and_use_only_neutral_fields`: pass.
- `rule_parser_accepts_supported_commands_and_rejects_bad_input`: pass.
- `repeated_fact_reinforces_one_row_without_duplicates`: pass, default theta.
- `empty_ticks_drive_allocated_field_quiet_under_a15c`: pass.
- `snapshot_roundtrip_is_byte_exact`: pass, default theta.
- `reused_row_rejects_stale_rowref_and_generation_mutations_are_logged`: pass, default theta.
- `freed_row_reuse_cannot_inherit_stale_w_edges`: pass, default theta.
- `tiny_sweep_writes_well_formed_csv`: pass.
- `script_parsing_maps_to_closed_action_enum_and_rejects_unknowns`: pass.
- `world_run_writes_jsonl_and_deterministic_dump_lines`: pass.
- `same_seeds_and_script_produce_byte_identical_jsonl`: pass.
- `fixed_script_matches_golden_world_hashes`: pass.
- `world_observations_and_runtime_paths_stay_quarantined`: pass.

World golden hashes did not change:

```text
world_golden_obs_hash=c0129c72e355a41f1b74ae2cadb07c8a82606d454c71011a640aadf417bc6229
world_golden_trace_hash=4f3b634b3ebd56231098d6fd0700a5a9e840b20a286579ff05bd9b46d162c699
```

Golden churn: none. S05 touched AM settle dynamics only; W0 world traces are unchanged.

## A15a Scaled Sweep

Command:

```text
$ /usr/bin/time -p cargo run --release -- sweep --grid sweep/grid.json --out sweep/results_s05_a15a.csv
   Compiling am001 v0.0.1 (/Users/anon5376/Desktop/AM001)
    Finished `release` profile [optimized] target(s) in 2.92s
     Running `target/release/am sweep --grid sweep/grid.json --out sweep/results_s05_a15a.csv`
wrote sweep/results_s05_a15a.csv total_points=1944 invalid_points=648 evaluated_points=1296 all_pass_points=45
real 24.73
user 20.82
sys 0.26
```

Grid:

- `lam_settle`: 0.2, 0.4, 0.55
- `beta`: 0.8, 1.0, 1.4
- `gamma`: 0.6, 0.9, 1.2
- `th0`: 1.0, 2.0, 3.0, 4.0
- `k_i`: 1.0, 2.0, 3.0
- `eta_w`: 0.05, 0.15, 0.3
- `del_w`: 0.0, 0.002
- `t`: 20
- scale: `n=512`, `completion_assemblies=12`

Counts:

```text
rows=1944
invalid=648
evaluated=1296
all_pass=45
determinism=1296
completion=93
reinforcement=474
contradiction=180
forgetting=1296
diff_integrity=1296
```

Best scaled candidate selected for full-scale confirmation:

```text
recall=1.000000 contamination=0.000000
theta_hash=035b5eb5a282f7b0eb273fa5032d59eec90a9962aa0841173a454cd54e92a104
n=512 assemblies=12 lam=0.55 beta=1.4 gamma=1.2 th0=3 k_i=3 eta_w=0.15 del_w=0
```

## Full-Scale Confirmation

Command:

```text
$ /usr/bin/time -p cargo run --release -- sweep --grid /tmp/am_s05_full_XXXX.json --out sweep/results_s05_full_confirm.csv
    Finished `release` profile [optimized] target(s) in 0.03s
     Running `target/release/am sweep --grid /tmp/am_s05_full_XXXX.json --out sweep/results_s05_full_confirm.csv`
wrote sweep/results_s05_full_confirm.csv total_points=1 invalid_points=0 evaluated_points=1 all_pass_points=1
real 1.02
user 0.11
sys 0.02
```

Full-scale row:

```text
dce727473be3778453afda3a214d6220ff8bca191a8ec731d151e9b301af3952,4096,30,true,0.55,1.4,1.2,3,3,0.15,0,0.4,0.01,0.6,0.05,0.25,20,true,true,true,true,true,true,true,1.000000,0.000000,""
```

A15b was not implemented because A15a produced all-pass points.
A15d was not implemented because no escalation was needed.

## Demo Transcript

Commands:

```text
$ cargo run --quiet -- init --snapshot data/am001_s05.bin
$ cargo run --quiet -- step-text --snapshot data/am001_s05.bin "assert am001 goal_relevance=1 agency=0.8 truth_assert=1"
$ cargo run --quiet -- step-text --snapshot data/am001_s05.bin "assert rust truth_assert=1 goal_relevance=0.8 effort=-0.3"
$ cargo run --quiet -- step-text --snapshot data/am001_s05.bin "link rust am001 1"
$ cargo run --quiet -- dump --snapshot data/am001_s05.bin --sort act --top 20
```

Output:

```text
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.00 b=0.05 cert=0.84 age=2
rust         [truth_assert +1.00  goal_relevance +0.80  effort -0.30]  a=0.00 b=0.05 cert=0.84 age=1
```

Contradiction sequence:

```text
$ cargo run --quiet -- step-text --snapshot data/am001_s05.bin "assert rust truth_assert=-1" --diff
a[rust]  0.000→0.388   ← decay lambda_a
u[rust]  2→4   ← assert activation
M[rust][truth_assert]  -0.166   ← write
V[rust][truth_assert]  +0.640   ← variance update
b[am001]  -0.000   ← baseline lambda_b
b[rust]  +0.004   ← baseline lambda_b

$ cargo run --quiet -- step-text --snapshot data/am001_s05.bin "assert rust truth_assert=1" --diff
a[rust]  0.388→0.389   ← decay lambda_a
u[rust]  4→5   ← assert activation
M[rust][truth_assert]  +0.014   ← write
V[rust][truth_assert]  -0.155   ← variance update
b[am001]  -0.000   ← baseline lambda_b
b[rust]  +0.004   ← baseline lambda_b

$ cargo run --quiet -- step-text --snapshot data/am001_s05.bin "assert rust truth_assert=-1" --diff
a[rust]  0.389→0.390   ← decay lambda_a
u[rust]  5→6   ← assert activation
M[rust][truth_assert]  -0.154   ← write
V[rust][truth_assert]  +0.444   ← variance update
b[rust]  +0.154   ← contradiction_open
contradiction[rust][truth_assert]   ← contradiction_open
b[am001]  -0.000   ← baseline lambda_b
```

Dump after contradiction:

```text
rust         [goal_relevance +0.80  truth_assert +0.69  effort -0.30]  a=0.39 b=0.22 cert=0.82 age=0  (min-axis cert: truth_assert 0.48)
   ⚠ contradiction open: truth_assert
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.00 b=0.05 cert=0.84 age=5
```

After 10 empty ticks:

```text
$ for i in 1 2 3 4 5 6 7 8 9 10; do cargo run --quiet -- step-text --snapshot data/am001_s05.bin "" >/dev/null; done
$ cargo run --quiet -- dump --snapshot data/am001_s05.bin --sort act --top 20
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.00 b=0.05 cert=0.84 age=15
rust         [goal_relevance +0.80  truth_assert +0.69  effort -0.30]  a=0.00 b=0.21 cert=0.82 age=10  (min-axis cert: truth_assert 0.48)
   ⚠ contradiction open: truth_assert
```

This proves the allocated field goes quiet after empty ticks.

## Files Changed

- `docs/AM_CORE_V0_AMENDMENTS.md`
- `docs/CODEX_WORKPLAN_S05.md`
- `docs/BUILD_REPORT_S05.md`
- `src/core/theta.rs`
- `src/core/resolve.rs`
- `src/core/settle.rs`
- `src/core/step.rs`
- `src/core/snapshot.rs`
- `src/core/state.rs`
- `src/eval/criteria.rs`
- `src/eval/sweep.rs`
- `src/cli/commands.rs`
- `sweep/grid.json`
- `sweep/results_s05_a15a.csv`
- `sweep/results_s05_full_confirm.csv`
- `tests/completion.rs`
- `tests/resting_field.rs`
- `tests/sweep_smoke.rs`
- deleted `tests/theta/completion.json`

## Deviations / Decisions

- Snapshot format is v4 because `Theta` gained `k_i`.
- A15b was not implemented because A15a produced full-scale all-pass default theta.
- A15d was not implemented because no escalation was needed.
- Historical reports still mention the old completion theta; they are retained as historical artifacts, not current truth.
- Generated demo snapshot `data/am001_s05.bin` is ignored and not intended as source.

## Known Limitations

- The scaled sweep is a filter, not proof; only the selected point was full-scale confirmed.
- The A15c invariant is a resting-bound guard, not a complete nonlinear stability proof for arbitrary high baselines.
- Completion now passes at full scale, but future changes to P2/P4 should rerun the full-scale confirmation row.

## Exact Next Tasks

1. Commit and push S05 once reviewed.
2. If future work changes settle, Hebb, or write gates, rerun `sweep/results_s05_full_confirm.csv` equivalent first.
3. Continue to S06 / M2 perception bridge only after preserving the A15c invariant and default theta hash in reports.
