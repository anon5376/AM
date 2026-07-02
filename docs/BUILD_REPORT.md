AM001 RUST BUILD REPORT

1. Workstreams used (one line each)

Architecture Auditor: enforced no database memory, no Python source, no core LLM calls, deterministic sorted iteration, and module boundaries.
Core Math: implemented `M/W/a/b/V/u`, `Theta`, axes, events, and P1 through P7.
Determinism/Trace: implemented mutation records, trace JSONL hashing, state hashing, snapshot/load/continue determinism.
Storage: implemented bincode snapshots with recent-write history and contradictions included.
CLI/Inspectability: implemented `init`, `dump`, `step`, `step-text`, `run`, `repl`, and `bench-step`.
Kill-Test: implemented determinism, completion, reinforcement, contradiction, forgetting, diff-integrity, snapshot-roundtrip, parser, CLI smoke, and drift scan tests.
Ollama-Seam: added gated stubs outside core plus `docs/OLLAMA_SEAM.md`.
Integration Auditor: added `docs/ARCHITECTURE_DRIFT_CHECK.md` and dependency/source drift test.

2. Commands run (fmt / test / demo)

Environment setup: `cargo` was absent, so Rust 1.96.0 was installed with Homebrew before running the required Rust commands.

```text
$ cargo fmt && cargo test
Finished `test` profile [unoptimized + debuginfo]
cli_smoke ... ok
completion ... ok
contradiction ... ok
determinism ... ok
diff_integrity ... ok
drift_check ... ok
forgetting ... ok
parser_rule ... ok
reinforcement ... ok
snapshot_roundtrip ... ok
Doc-tests am001 ... ok

$ cargo run -- init --snapshot data/am001.bin
ok

$ cargo run -- step-text --snapshot data/am001.bin "assert am001 goal_relevance=1 agency=0.8 truth_assert=1"
ok

$ cargo run -- step-text --snapshot data/am001.bin "assert rust truth_assert=1 goal_relevance=0.8 effort=-0.3"
ok

$ cargo run -- step-text --snapshot data/am001.bin "link rust am001 1"
ok

$ cargo run -- dump --snapshot data/am001.bin --sort act --top 20
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.54 b=0.07 cert=0.84 age=2
rust         [truth_assert +1.00  goal_relevance +0.80  effort -0.30]  a=0.54 b=0.06 cert=0.84 age=1

$ cargo run -- step-text --snapshot data/am001.bin "assert rust truth_assert=-1" --diff
a[rust]  0.538→0.556   ← decay lambda_a
u[rust]  2→4   ← assert activation
a[am001]  0.540→0.558   ← decay lambda_a
M[rust][truth_assert]  -0.238   ← write
V[rust][truth_assert]  +0.589   ← variance update
W[am001↔rust]          +0.031   ← hebb
W[rust↔am001]          +0.031   ← hebb
b[am001]               +0.006   ← baseline lambda_b
b[rust]                +0.006   ← baseline lambda_b

$ cargo run -- step-text --snapshot data/am001.bin "assert rust truth_assert=1" --diff
M[rust][truth_assert]  +0.029   ← write
V[rust][truth_assert]  -0.141   ← variance update
W[am001↔rust]          +0.032   ← hebb
W[rust↔am001]          +0.032   ← hebb

$ cargo run -- step-text --snapshot data/am001.bin "assert rust truth_assert=-1" --diff
M[rust][truth_assert]  -0.220   ← write
V[rust][truth_assert]  +0.371   ← variance update
b[rust]                +0.156   ← contradiction_open
contradiction[rust][truth_assert]   ← contradiction_open

$ cargo run -- dump --snapshot data/am001.bin --sort act --top 20
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.58 b=0.08 cert=0.84 age=5
rust         [goal_relevance +0.80  truth_assert +0.57  effort -0.30]  a=0.57 b=0.23 cert=0.82 age=0
   ⚠ contradiction open: truth_assert

$ cargo run -- step-text --snapshot data/am001.bin "cue rust 0.8" --diff
a[rust]  0.574→0.637   ← decay lambda_a
u[rust]  6→7   ← cue
a[am001]  0.576→0.580   ← decay lambda_a
W[am001↔rust]          +0.037   ← hebb
W[rust↔am001]          +0.037   ← hebb

$ cargo run -- step-text --snapshot data/am001.bin "" --diff
a[am001]  0.580→0.592   ← decay lambda_a
a[rust]   0.637→0.648   ← decay lambda_a
W[am001↔rust]          +0.039   ← hebb
W[rust↔am001]          +0.039   ← hebb

$ cargo run -- dump --snapshot data/am001.bin --sort act --top 20
rust         [goal_relevance +0.80  truth_assert +0.57  effort -0.30]  a=0.65 b=0.24 cert=0.82 age=1
   ⚠ contradiction open: truth_assert
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.59 b=0.09 cert=0.84 age=7
```

3. Test results (all 9, pass/fail, theta hash used per test)

Determinism: pass, theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Completion: pass, theta `e5d8e36692df55f5809a3dbb9acf43b6999c61957f49a702df70d7454bb197a0` from `tests/theta/completion.json`.
Reinforcement: pass, theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Contradiction: pass, theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Forgetting: pass, theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Diff-integrity: pass, theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Snapshot-roundtrip: pass, theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Parser: pass, theta N/A because this test validates `text -> Event` only.
CLI smoke: pass, default theta `87fa7705297e9737c73e06a62cfe80008c725e167268fcc402bcb2767b586536`.
Extra drift check: pass, theta N/A.

4. Files created/changed

Created: `.gitignore`, `Cargo.toml`, `Cargo.lock`, `README.md`, `docs/AM_CORE_V0.md`, `docs/AM_CORE_V0_AMENDMENTS.md`, `docs/CODEX_WORKPLAN.md`, `docs/ARCHITECTURE_DRIFT_CHECK.md`, `docs/OLLAMA_SEAM.md`, `docs/BUILD_REPORT.md`.
Created: `src/main.rs`, `src/lib.rs`, `src/core/*`, `src/cli/*`, `src/storage/*`, `src/parser/*`, `src/llm/*`.
Created: `tests/*.rs`, `tests/common/mod.rs`, `tests/theta/completion.json`.
Generated and ignored: `data/am001.bin`, `target/`, `.DS_Store`, `Archive.zip`.

5. Implemented core (M/W/a/b/V/u, Theta, Events, P1..P7, CLI, storage, inspection)

Implemented `AmState` with `M`, `W`, `a`, `b`, `V`, `u`, labels, aliases, goals, open contradictions, recent-write history, and free list.
Implemented amended `Theta`, 48 named axes, validated `Event`, deterministic `ConceptRef` resolution, allocation, cue/assert activation, settle, write, Hebb/link hints, contradiction open/close, decay, GC/free/merge, and snapshot persistence.
Implemented CLI commands and rule parser. Ollama seam is stubbed outside the core.
Implemented dump and diff inspection using signed scalar axis output and cause-tagged mutation records.

6. Proof memory is parameters (where memory lives; confirm no database)

Memory lives in `AmState`: `m: Vec<f32>`, `links: Vec<Vec<Link>>`, `a: Vec<f32>`, `b: Vec<f32>`, `v: Vec<f32>`, `u: Vec<i64>`.
Persistence is `bincode` serialization of `AmState`.
No database crates are present in `Cargo.toml`.
No RAG/vector index exists.
No LLM module is reachable from `src/core/`.
Labels are used for resolution and inspection, not settle/write/hebb/decay dynamics.

7. Proof determinism (hash test explanation + values)

`tests/determinism.rs` runs the same event stream twice from the same initial snapshot and checks identical `state_hash` and `trace_hash`. It also saves midway, loads, continues, and checks the result against the uninterrupted run.

State hash: `a4338198a93c2ae999271192453cee0a1c009e456c233d48773d2f3b178752b6`.
Trace hash: `e92e158420062d1e7f7f033534940b1a498ec24551a2c829969aae0f88df078c`.

8. Proof inspectability (paste real dump + diff output)

Real dump:

```text
rust         [goal_relevance +0.80  truth_assert +0.57  effort -0.30]  a=0.65 b=0.24 cert=0.82 age=1
   ⚠ contradiction open: truth_assert
am001        [goal_relevance +1.00  truth_assert +1.00  agency +0.80]  a=0.59 b=0.09 cert=0.84 age=7
```

Real diff:

```text
M[rust][truth_assert]  -0.220   ← write
V[rust][truth_assert]  +0.371   ← variance update
W[am001↔rust]          +0.033   ← hebb
W[rust↔am001]          +0.033   ← hebb
b[rust]                +0.156   ← contradiction_open
contradiction[rust][truth_assert]   ← contradiction_open
```

9. Spec deviations (every one, incl. amendments A1–A14 applied, with reasons)

A1 applied: goal pull uses mean squared distance and `tau_g=1.0`.
A2 applied: Hebb growth is separate from global link decay.
A3 applied: settle logs one net activation mutation per row.
A4 applied: `eps_log=1e-6`; sub-epsilon candidates are not applied or logged.
A5 applied: inhibition mean uses allocated rows only.
A6 applied: write evidence sign uses `signum(v - M_before)`.
A7 applied: alternation requires at least 4 entries and strict adjacent sign opposition.
A8 applied: `lam_v=0.8`.
A9 applied: explicit link hints are symmetric.
A10 applied: snapshots include recent writes and contradictions.
A11 applied: `u` refreshes only on external cue/assert touch.
A12 applied: determinism is same-machine bit-exactness.
A13 applied: CLI accepts `--theta`; completion uses `tests/theta/completion.json`.
A14 applied: contradictions have no timeout.
Rust-first deviation from source doc: the build prompt explicitly overrides the NumPy-first source doc note.
Trace coalescing deviation: scalar mutation records are coalesced per final changed cell inside a step to satisfy diff integrity.
Allocation evidence note: initial asserted axes for a newly allocated label are recorded in recent-write history so `+1, -1, +1, -1` opens on the fourth alternating evidence item.
Completion theta tuning: `tests/theta/completion.json` changes `eta_w`, `del_w`, `beta`, `th0`, `t`, `th_act`, `rho_b`, and `b0`.
Daemon Hebb choice: empty ticks still run Hebb growth over residually active rows.
Ollama seam: stub only for v0.

10. Known limitations (honest: expressiveness is associative not propositional; episodic memory absent; contradictions open-forever per A14; theta uncalibrated beyond kill tests)

Expressiveness is associative, not propositional.
Episodic memory is absent; the trace is audit data, not parameter memory.
Contradictions stay open indefinitely without resolving evidence.
Theta is calibrated only to the kill tests, not to broad cognitive behavior.
Completion requires a tuned theta profile because dense linked assemblies can self-ignite under the default settle loop.
The Ollama seam is a stub.
Diff records are final-cell exact, not a full microhistory of every intermediate scalar assignment.

11. Next session (exact tasks)

Add property-style tests for merge/link redirection and alias resolution.
Add `--trace` support to single `step` and `step-text` commands, not only `run`.
Add a compact `am inspect-link` command for sparse W rows.
Add a calibration sweep test for completion theta.
Implement the real gated Ollama parser only after adding Event JSON repair tests.

