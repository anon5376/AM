# AM001 S03 Build Report

## Workstreams

- Allocation hardening: added explicit `allocated: Vec<bool>` parameter state; `is_allocated()` and row iteration no longer depend on labels or the free list.
- W/free/merge hardening: kept `Link.target: usize` (option B) and proved stale links are removed before reuse; every W removal/redirection goes through logged mutation-site helpers.
- RowRef/generation audit: stale external `RowRef` rejection remains pre-tick/pre-mutation and is covered by byte-identical snapshot test.
- W eps_log audit: W decay is not applied when `abs(delta) < eps_log`; W records are coalesced to exactly one final changed cell.
- Diff/drift hardening: expanded diff integrity and architecture drift checks for allocation, W/free/merge/reuse, label-free dynamics, dormant Ollama/core clients, no Python, no core rand/thread/time, and semantic quarantine.
- Snapshot layout: bumped `SNAPSHOT_FORMAT_VERSION` from 2 to 3 because S03 adds serialized allocation state.

## Commands Run

```text
$ cargo fmt --check
```

Result: exit 0, no stdout.

```text
$ cargo clippy
    Checking am001 v0.0.1 (/Users/anon5376/Desktop/AM001)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.79s
```

```text
$ cargo test -- --nocapture
test cli_init_step_text_and_dump_smoke ... ok
test partial_cue_completes_linked_assemblies_without_foreign_spill ... ok
test alternating_evidence_opens_and_same_sign_evidence_closes ... ok
test fixed_stream_and_midway_snapshot_are_deterministic ... ok
test diff_integrity_covers_contradiction_free_reuse_merge_and_sub_eps_w_decay ... ok
test every_changed_scalar_and_link_has_exactly_one_trace_record ... ok
test architecture_drift_scan_rejects_forbidden_dependencies_and_python ... ok
test stale_low_baseline_rows_free_merge_and_protected_rows_survive ... ok
test dump_shows_min_axis_certainty_and_axes_lists_all_axes ... ok
test snapshot_version_mismatch_is_refused_clearly ... ok
test rule_parser_accepts_supported_commands_and_rejects_bad_input ... ok
test repeated_fact_reinforces_one_row_without_duplicates ... ok
test snapshot_roundtrip_is_byte_exact ... ok
test reused_row_rejects_stale_rowref_and_generation_mutations_are_logged ... ok
test freed_row_reuse_cannot_inherit_stale_w_edges ... ok
test tiny_sweep_writes_well_formed_csv ... ok
Doc-tests am001: ok
```

Full test result: 16 integration tests passed, 0 failed; lib/main/doc tests had 0 tests and passed.

Theta hashes printed by tests:

- Default theta: `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`
- Completion theta: `e5d8e36692df55f5809a3dbb9acf43b6999c61957f49a702df70d7454bb197a0`

Stale-row/W-link demo lines from `cargo test -- --nocapture`:

```text
stale_rowref_demo: stale_ref=0:1 current_ref=0:3 tick_unchanged=3
stale_w_links_demo: freed_row=1 reused_row=1 removed_w_records=2 c_active_after_cue=0.000000
```

## Manual Demo

```text
$ cargo run -- init --snapshot data/am001_s03.bin
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.53s
     Running `target/debug/am init --snapshot data/am001_s03.bin`
```

```text
$ cargo run -- step-text --snapshot data/am001_s03.bin "assert rust truth_assert=1 goal_relevance=0.8" --diff
generation[rust]  0→1   ← row generation
alloc[rust]   ← allocate
label[rust]   ← allocate
...
M[rust][goal_relevance]  +0.800   ← allocate
M[rust][truth_assert]  +1.000   ← allocate
a[rust]  0.000→0.531   ← decay lambda_a
```

```text
$ cargo run -- step-text --snapshot data/am001_s03.bin "assert rust truth_assert=-1" --diff
a[rust]  0.531→0.532   ← decay lambda_a
u[rust]  1→2   ← assert activation
M[rust][truth_assert]  -0.228   ← write
V[rust][truth_assert]  +0.596   ← variance update
b[rust]  +0.005   ← baseline lambda_b
```

```text
$ cargo run -- step-text --snapshot data/am001_s03.bin "assert rust truth_assert=1" --diff
a[rust]  0.532→0.533   ← decay lambda_a
u[rust]  2→3   ← assert activation
M[rust][truth_assert]  +0.026   ← write
V[rust][truth_assert]  -0.143   ← variance update
b[rust]  +0.005   ← baseline lambda_b
```

```text
$ cargo run -- step-text --snapshot data/am001_s03.bin "assert rust truth_assert=-1" --diff
a[rust]  0.533→0.535   ← decay lambda_a
u[rust]  3→4   ← assert activation
M[rust][truth_assert]  -0.206   ← write
V[rust][truth_assert]  +0.384   ← variance update
b[rust]  +0.155   ← contradiction_open
contradiction[rust][truth_assert]   ← contradiction_open
```

```text
$ cargo run -- dump --snapshot data/am001_s03.bin --sort act --top 10
rust         [goal_relevance +0.80  truth_assert +0.59]  a=0.53 b=0.22 cert=0.82 age=0  (min-axis cert: truth_assert 0.50)
   ⚠ contradiction open: truth_assert
```

## Full Axes Output

```text
$ cargo run -- axes --snapshot data/am001_s03.bin rust
rust  row=0 gen=1 cert=0.823
urgency                      value=+0.0000 V=0.2000 cert=0.8333
valence                      value=+0.0000 V=0.2000 cert=0.8333
arousal                      value=+0.0000 V=0.2000 cert=0.8333
agency                       value=+0.0000 V=0.2000 cert=0.8333
self_relevance               value=+0.0000 V=0.2000 cert=0.8333
user_relevance               value=+0.0000 V=0.2000 cert=0.8333
goal_relevance               value=+0.8000 V=0.1600 cert=0.8621
temporal_near                value=+0.0000 V=0.2000 cert=0.8333
temporal_past                value=+0.0000 V=0.2000 cert=0.8333
concreteness                 value=+0.0000 V=0.2000 cert=0.8333
social                       value=+0.0000 V=0.2000 cert=0.8333
risk                         value=+0.0000 V=0.2000 cert=0.8333
effort                       value=+0.0000 V=0.2000 cert=0.8333
value                        value=+0.0000 V=0.2000 cert=0.8333
novelty                      value=+0.0000 V=0.2000 cert=0.8333
stability                    value=+0.0000 V=0.2000 cert=0.8333
truth_assert                 value=+0.5920 V=0.9972 cert=0.5007
desire                       value=+0.0000 V=0.2000 cert=0.8333
obligation                   value=+0.0000 V=0.2000 cert=0.8333
completion                   value=+0.0000 V=0.2000 cert=0.8333
familiarity                  value=+0.0000 V=0.2000 cert=0.8333
emotional_charge             value=+0.0000 V=0.2000 cert=0.8333
scope                        value=+0.0000 V=0.2000 cert=0.8333
priority                     value=+0.0000 V=0.2000 cert=0.8333
architecture_relevance       value=+0.0000 V=0.2000 cert=0.8333
memory_relevance             value=+0.0000 V=0.2000 cert=0.8333
language_relevance           value=+0.0000 V=0.2000 cert=0.8333
implementation_relevance     value=+0.0000 V=0.2000 cert=0.8333
contradiction_relevance      value=+0.0000 V=0.2000 cert=0.8333
uncertainty_relevance        value=+0.0000 V=0.2000 cert=0.8333
power_relevance              value=+0.0000 V=0.2000 cert=0.8333
autonomy                     value=+0.0000 V=0.2000 cert=0.8333
tool_relevance               value=+0.0000 V=0.2000 cert=0.8333
reasoning_relevance          value=+0.0000 V=0.2000 cert=0.8333
planning_relevance           value=+0.0000 V=0.2000 cert=0.8333
identity_relevance           value=+0.0000 V=0.2000 cert=0.8333
preference_relevance         value=+0.0000 V=0.2000 cert=0.8333
constraint_relevance         value=+0.0000 V=0.2000 cert=0.8333
project_relevance            value=+0.0000 V=0.2000 cert=0.8333
learning_relevance           value=+0.0000 V=0.2000 cert=0.8333
recency                      value=+0.0000 V=0.2000 cert=0.8333
persistence                  value=+0.0000 V=0.2000 cert=0.8333
confidence_proxy             value=+0.0000 V=0.2000 cert=0.8333
activation_bias              value=+0.0000 V=0.2000 cert=0.8333
attention                    value=+0.0000 V=0.2000 cert=0.8333
specificity                  value=+0.0000 V=0.2000 cert=0.8333
context_relevance            value=+0.0000 V=0.2000 cert=0.8333
safety_relevance             value=+0.0000 V=0.2000 cert=0.8333
```

## Benchmark

```text
$ cargo run -- bench-step --snapshot data/am001_s03_bench.bin --events 1000
bench-step events=1000 mean_us=361 max_us=615
```

## Sweep / Unified Theta

Debug-mode sweep was accidentally started first:

```text
$ cargo run -- sweep --grid sweep/grid.json --out sweep/results_s03_check.csv
```

It was interrupted after 20m39s because it stayed CPU-bound and wrote no CSV. The same grid was then run correctly in release mode:

```text
$ cargo run --release -- sweep --grid sweep/grid.json --out sweep/results_s03_check.csv
wrote sweep/results_s03_check.csv
```

`sweep/results_s03_check.csv` is byte-identical to `sweep/results.csv`.

No uniform theta was found in the requested 729-point grid

Grid summary:

- rows: 729
- all-pass: 0
- determinism pass: 729
- completion pass: 0
- reinforcement pass: 567
- contradiction pass: 405
- forgetting pass: 729
- diff-integrity pass: 729

Best three grid points by completion recall then contamination:

```text
0.433333,0.966667,0434f7f8f3bbb70e80271358ccdc534fc6c0f94c897e7990669327c858538db0,lam_settle=0.2,beta=1.8,gamma=0.6,th0=2,eta_w=0.15,del_w=0.002
0.433333,0.966667,04fb8adce1a91266ee991ecc490aa9a58cc5f68f31e5c9330e3d16f8282e03de,lam_settle=0.55,beta=1,gamma=0.6,th0=2,eta_w=0.15,del_w=0
0.433333,0.966667,07086c8eeeca79b2a5ff7b0f00a979ce664332e50fb3feb1354db78a8b4d60c7,lam_settle=0.35,beta=1.4,gamma=0.6,th0=2,eta_w=0.15,del_w=0.001
```

Default theta remains unchanged. `tests/theta/completion.json` remains because the default theta still does not pass completion at t=20.

## Test Results By Criterion

- Determinism: pass, default theta hash `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
- Completion: pass only with `tests/theta/completion.json`, theta hash `e5d8e36692df55f5809a3dbb9acf43b6999c61957f49a702df70d7454bb197a0`.
- Reinforcement: pass, default theta.
- Contradiction: pass, default theta.
- Forgetting/free/merge/protected rows: pass, default theta.
- Diff-integrity: pass, default theta; now covers allocation, contradiction open/close, free outgoing/inbound W, reuse, merge redirection/removal, sub-eps W decay, and snapshot roundtrip after free/merge/reuse.
- Snapshot mismatch refusal: pass; format is now v3.
- Snapshot roundtrip: pass.
- Stale RowRef: pass; stale explicit `RowRef` rejects before tick/mutation and preserves byte-identical snapshot.
- Stale W links: pass; freed row reused at same index without inherited inbound/outgoing W.
- Sweep smoke: pass.
- Drift/semantic lint: pass.
- Parser and CLI smoke: pass.

## Files Changed In S03

- Added `docs/CODEX_WORKPLAN_S03.md`.
- Added `docs/BUILD_REPORT_S03.md`.
- Added `tests/stale_w_links.rs`.
- Changed `docs/ARCHITECTURE_DRIFT_CHECK.md`.
- Changed `src/core/state.rs`, `src/core/trace.rs`, `src/core/resolve.rs`, `src/core/hebb.rs`, `src/core/decay.rs`, `src/core/snapshot.rs`.
- Changed `tests/diff_integrity.rs`, `tests/drift_check.rs`, `tests/stale_rowref.rs`.
- Added fresh sweep output `sweep/results_s03_check.csv`.

The working tree also still contains S02 uncommitted files from the prior pass (`src/eval/*`, `sweep/results.csv`, `docs/BUILD_REPORT_S02.md`, and related tests/docs).

## Deviations / Decisions

- Chose option B for sparse W: `Link.target` remains `usize`. Exception rationale: W is internal slot-index state, not external identity; S03 now proves every outgoing and inbound link to a freed row is removed and logged before slot reuse.
- Rejected events do not emit a trace record. Rationale: explicit stale `RowRef` validation runs before `tick += 1`; no tick, mutation, or trace exists for a rejected event. `tests/stale_rowref.rs` proves the snapshot is byte-identical after rejection.
- Snapshot format is v3, not v2, because adding `allocated` changed serialized state layout after S02.
- The first S03 full sweep attempt was interrupted in dev mode after 20m39s; release-mode sweep completed and produced the current CSV.
- Manual demo allocation diff is abbreviated above because it emits all 48 initial V rows; the full axes output is included without ellipsis as required.

## Known Limitations

- No uniform theta passing all requested criteria at t=20 was found in the 729-point grid.
- Completion still uses `tests/theta/completion.json`.
- W free cleanup scans all rows for inbound links; this is deterministic and safe, but still O(N) on free/merge. Acceptable for AM001.1 hardening; future sparse reverse indexes would need the same mutation/diff guarantees.
- No M1/W0 world harness, observation JSON/action enum, closed effect/relation enums, schemas, tracks, or episodic buffer were built in S03.

## Exact Next Tasks

1. Decide whether completion should be retrained by event design rather than theta alone, then rerun the unified-theta audit.
2. Add a release-mode sweep command note to README so future full-grid checks do not run in debug mode.
3. If free/merge becomes hot in world harness runs, introduce a deterministic reverse-W index with full MutationRecord/diff-integrity coverage.
4. Start M1/W0 only after committing this hardening baseline.
