AM001 S02 BUILD REPORT

1. Workstreams used

Architecture Auditor: enforced L1-L24, added semantic-quarantine lint, and kept Ollama dormant.
Core State: added snapshot v2 envelope, `format_version`, row generation counters, and `RowRef { id, gen }`.
Eval/Sweep: moved core kill criteria into `src/eval/criteria.rs` and added `am sweep`.
Inspectability: added min per-axis certainty in `dump` and full `am axes <label>`.
CLI/Performance: made `bench-step` print mean/max step latency.
Test: added sweep smoke, stale rowref, snapshot-format refusal, dump/axes, and generation diff-integrity coverage.

2. Commands run

```text
cargo fmt && cargo clippy && cargo test
```

Result: pass, no clippy warnings.

```text
target/release/am sweep --grid sweep/grid.json --out sweep/results.csv
```

Result: `sweep/results.csv`, 729 points, 0 all-pass points.

Demo:

```text
$ cargo run -- init --snapshot data/am001_s02.bin
$ cargo run -- step-text --snapshot data/am001_s02.bin "assert am001 goal_relevance=1 agency=0.8 truth_assert=1"
$ cargo run -- step-text --snapshot data/am001_s02.bin "assert rust truth_assert=1 goal_relevance=0.8 effort=-0.3"
$ cargo run -- step-text --snapshot data/am001_s02.bin "link rust am001 1"
$ cargo run -- dump --snapshot data/am001_s02.bin --sort act --top 20
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.54 b=0.07 cert=0.84 age=2
rust         [truth_assert +1.00  goal_relevance +0.80  effort -0.30]  a=0.54 b=0.06 cert=0.84 age=1

$ cargo run -- step-text --snapshot data/am001_s02.bin "assert rust truth_assert=-1" --diff
M[rust][truth_assert]  -0.238   ← write
V[rust][truth_assert]  +0.589   ← variance update
W[am001↔rust]  +0.031   ← hebb
W[rust↔am001]  +0.031   ← hebb

$ cargo run -- step-text --snapshot data/am001_s02.bin "assert rust truth_assert=1" --diff
M[rust][truth_assert]  +0.029   ← write
V[rust][truth_assert]  -0.141   ← variance update

$ cargo run -- step-text --snapshot data/am001_s02.bin "assert rust truth_assert=-1" --diff
M[rust][truth_assert]  -0.220   ← write
V[rust][truth_assert]  +0.371   ← variance update
b[rust]  +0.156   ← contradiction_open
contradiction[rust][truth_assert]   ← contradiction_open

$ cargo run -- dump --snapshot data/am001_s02.bin --sort act --top 20
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.58 b=0.08 cert=0.84 age=5
rust         [goal_relevance +0.80  truth_assert +0.57  effort -0.30]  a=0.57 b=0.23 cert=0.82 age=0  (min-axis cert: truth_assert 0.51)
   ⚠ contradiction open: truth_assert

$ cargo run -- axes --snapshot data/am001_s02.bin rust
rust  row=1 gen=1 cert=0.823
truth_assert                 value=+0.5700 V=0.9791 cert=0.5053
... all 48 axes printed ...

$ cargo run -- step-text --snapshot data/am001_s02.bin "cue rust 0.8" --diff
a[rust]  0.574→0.637   ← decay lambda_a
u[rust]  6→7   ← cue
a[am001]  0.576→0.580   ← decay lambda_a
W[am001↔rust]  +0.037   ← hebb
W[rust↔am001]  +0.037   ← hebb

$ cargo run -- step-text --snapshot data/am001_s02.bin "" --diff
a[am001]  0.580→0.592   ← decay lambda_a
a[rust]  0.637→0.648   ← decay lambda_a
W[am001↔rust]  +0.039   ← hebb
W[rust↔am001]  +0.039   ← hebb

$ cargo run -- dump --snapshot data/am001_s02.bin --sort act --top 20
rust         [goal_relevance +0.80  truth_assert +0.57  effort -0.30]  a=0.65 b=0.24 cert=0.82 age=1  (min-axis cert: truth_assert 0.51)
   ⚠ contradiction open: truth_assert
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.59 b=0.09 cert=0.84 age=7

$ cargo run -- bench-step --snapshot data/am001_s02.bin --events 1000
bench-step events=1000 mean_us=690 max_us=885
```

3. Test results with theta hashes

Default theta hash: `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Completion theta hash: `e5d8e36692df55f5809a3dbb9acf43b6999c61957f49a702df70d7454bb197a0`.

Determinism: pass, default theta.
Completion: pass, `tests/theta/completion.json`.
Reinforcement: pass, default theta.
Contradiction: pass, default theta.
Forgetting: pass, default theta.
Diff-integrity: pass, default theta, now covers generation mutations.
Snapshot-roundtrip: pass, default theta.
Parser: pass, theta N/A.
CLI smoke: pass, default theta.
Sweep smoke: pass, default theta grid point.
Stale rowref: pass, default theta.
Dump/axes snapshot-format: pass, default theta.
Drift/semantic lint: pass, theta N/A.

4. Files changed

Added: `docs/AM_ROADMAP_V2.md`, `docs/CODEX_WORKPLAN_S02.md`, `docs/BUILD_REPORT_S02.md`.
Added: `src/eval/mod.rs`, `src/eval/criteria.rs`, `src/eval/sweep.rs`.
Added: `sweep/grid.json`, `sweep/results.csv`.
Added: `tests/sweep_smoke.rs`, `tests/stale_rowref.rs`, `tests/inspect_snapshot.rs`.
Changed: core state/event/snapshot/trace/resolve/settle/contradiction/decay/inspect/step modules.
Changed: CLI commands, lib module exports, drift/diff/contradiction/forgetting tests.

5. Unified theta result

No uniform theta was found in the requested grid.

Grid:

- `lam_settle`: 0.2, 0.35, 0.55
- `beta`: 1.0, 1.4, 1.8
- `gamma`: 0.6, 0.9, 1.2
- `th0`: 0.5, 1.0, 2.0
- `eta_w`: 0.05, 0.15, 0.3
- `del_w`: 0.0, 0.001, 0.002
- `t`: 20

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
0434f7f8f3bbb70e80271358ccdc534fc6c0f94c897e7990669327c858538db0 recall=0.433333 contamination=0.966667 lam_settle=0.2 beta=1.8 gamma=0.6 th0=2 eta_w=0.15 del_w=0.002
04fb8adce1a91266ee991ecc490aa9a58cc5f68f31e5c9330e3d16f8282e03de recall=0.433333 contamination=0.966667 lam_settle=0.55 beta=1 gamma=0.6 th0=2 eta_w=0.15 del_w=0
07086c8eeeca79b2a5ff7b0f00a979ce664332e50fb3feb1354db78a8b4d60c7 recall=0.433333 contamination=0.966667 lam_settle=0.35 beta=1.4 gamma=0.6 th0=2 eta_w=0.15 del_w=0.001
```

Outside-grid probe: high `th0` plus stronger `eta_w` can make completion pass at t=20, but then asserted rows fail to stay active enough for contradiction. Lowering `th_act` restores contradiction and reintroduces completion contamination. That is the audited conflict, not a missing CLI flag.

Action taken: default theta is unchanged; `tests/theta/completion.json` is kept.

6. Snapshot versioning + row generations

Snapshot files now serialize a v2 envelope with `format_version: 2`.
`AmState` carries `format_version` and `generation: Vec<u32>`.
`ConceptRef::Id` is now `ConceptRef::Id(RowRef { id, gen })`.
Explicit stale `RowRef`s are preflighted before tick mutation and rejected with a clear stale-row-ref error.
Generation increments are logged with `MutationTarget::Generation` and `Cause::RowGeneration`.
Version mismatch is refused by `from_bytes`; no v1 migrator is provided.

7. Per-axis certainty

`dump` keeps row-mean cert and appends a min-axis cert note when `V > th_v` or an open contradiction exists.
`am axes <label>` prints all 48 axes with coordinate value, V, and per-axis certainty.

8. Deviations with reasons

No phase formulas were changed.
No completion threshold was weakened.
No uniform theta was committed because the requested grid produced zero all-pass points.
Sweep supports extra optional fields (`th_write`, `rho_b`, `a_init`, `b0`, `th_act`) for audit probes; the requested `sweep/grid.json` uses only the required fields.
Snapshot v1 is refused rather than migrated; that is intentional because v2 adds generation semantics.
The root `AM_ROADMAP_v2_1.md` is left untouched; canonical copy is `docs/AM_ROADMAP_V2.md`.

9. Known limitations

Completion still relies on a tuned theta file.
The unified-theta conflict remains unresolved at t=20.
CSV sweep is deterministic but slow because it runs real criteria.
RowRef protects external/stored row ids; sparse W still stores row indices internally and depends on GC/link cleanup.
No world harness, observation vocabulary, action enum, typed relations, schemas, or planner were built in S02.

10. Exact next tasks

Decide whether the completion criterion should train via pure co-cueing/co-asserting instead of explicit link hints, then rerun the theta audit.
Add a faster release-mode criterion runner or criterion-level caching for sweeps.
Investigate separating assert/write activation from associative settle activation so high-th0 completion does not starve contradiction writes.
Add v1 snapshot migration only if old local snapshots matter.
Start M1 world harness with semantic-quarantine lint active from day one.

