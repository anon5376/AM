# AM001 Track B Night 2 Build Report

## B0 — Apply Rejection Exit Fix

### Workstream

- Fixed `am apply` normal rejection exits so they no longer bubble an `anyhow::Error` out of `main`.
- Rejection outcomes now print exactly one stderr line and call `std::process::exit(1)`:
  - `EG-1 file rejected before apply`
  - `EG-1 file has structural rejection`
  - `EG-1 apply completed with rejected events`
- Extended `apply_rejection_exit_prints_one_clean_error_line` to assert the same one-line stderr behavior with `RUST_BACKTRACE` unset and with `RUST_BACKTRACE=1`.

### Commands

```text
$ cargo fmt && cargo test --test beval_b02 apply_rejection_exit_prints_one_clean_error_line -- --nocapture
test apply_rejection_exit_prints_one_clean_error_line ... ok
test result: ok. 1 passed; 0 failed
```

```text
$ cargo fmt --check && cargo clippy -- -D warnings && cargo test
69 tests passed; 0 failed
```

### Files Changed

- `src/cli/commands.rs`
- `tests/beval_b02.rs`
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

None.

### Known Limitations

None for B0.

### Next

B03 context compiler.

## B03 — Compiled AM Context and B2 Lane

### Workstream

- Added `src/beval/compile.rs` and `am compile-context --snapshot <p> [--budget 1200] --out <f>`.
- Compiled context sections render in fixed order: `STATE`, `ACTIVE`, `LINKS`, `CONTRADICTIONS`, `LOW-CERT`, `GOALS`, `RECENT`, plus the mandatory contradiction directive when open contradictions exist.
- Budgeting uses the pinned char/4 token estimate and removes whole entries in priority order: `RECENT`, `LOW-CERT`, `LINKS`, `ACTIVE`; fixed sections are never truncated.
- Wired lane B2 to compiled snapshot context. B2 now requires `--snapshot`; it no longer skips as `LaneUnavailable`.
- Added B03 goldens at 1200 and 400 tokens and added 29 B2 synthetic replay fixtures over the frozen B02 corpus.

### Commands

```text
$ cargo test --test beval_b03 -- --nocapture
running 6 tests
test compile_context_budget_and_truncation_order ... ok
test contradiction_directive_presence_and_absence ... ok
test compile_context_golden_byte_identity ... ok
test b2_lane_replay_scores_synthetic_fixtures ... ok
test full_beval_replay_double_run_byte_identical_b0_b1_b2 ... ok
test compile_context_cli_writes_budgeted_output ... ok
test result: ok. 6 passed; 0 failed
```

```text
$ cargo test --test beval_b02 -- --nocapture
running 9 tests
test b2_lane_requires_snapshot ... ok
test replay_double_run_byte_identical_b0_b1 ... ok
test apply_rejection_exit_prints_one_clean_error_line ... ok
test result: ok. 9 passed; 0 failed
```

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
71 tests passed; 0 failed
```

### Demo

```text
$ am compile-context --snapshot <b03-demo.bin> --budget 400 --out <context.txt>
compile-context out=<context.txt> tokens=223
STATE:
theta_hash 6905c5f8beede6bd3dfd9fc403626eafb433488291667610d11f0db54f23520c
tick 20
row_count 5
...
CONTRADICTIONS:
cpt_1 stability (min-axis cert 0.47)
cpt_2 architecture_relevance (min-axis cert 0.48)
cpt_3 completion (min-axis cert 0.48)
cpt_4 persistence (min-axis cert 0.48)
cpt_5 reasoning_relevance (min-axis cert 0.48)
...
DIRECTIVE:
OPEN CONTRADICTIONS EXIST. Do not present affected concepts as settled. Ask for a test or propose disambiguation.
```

```text
$ am beval --corpus beval/corpus --lane b2 --transport replay --fixtures beval/fixtures/synthetic_v1 --snapshot <b03-demo.bin> --out <b2.json>
beval lane=b2 transport=replay evaluated=29 skipped=0 out=<b2.json>
category scores: stage_accuracy=8/8, stale_claim=5/5, contradiction_handling=5/5, test_grounded=5/5, drift=6/6
total_context_tokens=15457
```

### Hashes

```text
$ shasum -a 256 tests/golden/b03_context_1200.txt tests/golden/b03_context_400.txt beval/fixtures/synthetic_v1/*.txt | shasum -a 256
9521a8662d7d8b953b86a0ad50519ac11246df97bb05bae7db4173cad24ba27b  -
```

### Files Changed

- `src/beval/compile.rs`
- `src/beval/mod.rs`
- `src/beval/prompt.rs`
- `src/cli/commands.rs`
- `tests/beval_b02.rs`
- `tests/beval_b03.rs`
- `tests/golden/b03_context_1200.txt`
- `tests/golden/b03_context_400.txt`
- `beval/fixtures/synthetic_v1/*.txt` (29 B2 prompt fixtures)
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

- `RECENT` is compiled from snapshot-resident `recent_writes` evidence as a deduplicated write-cause count (`write xN`). The snapshot does not contain the full external trace log or complete per-step mutation-cause history, and B03 did not add trace-memory state to the core.
- The frozen B02 corpus/matchers were not edited. This means `stale_b2_ready` still tests the B02-era stale claim by design while B03 proves the new B2 lane separately.

### Known Limitations

- Synthetic replay fixtures prove deterministic prompt hashing, context budgeting, replay, and scoring. They are not model-quality evidence.

### Next

B04 ingest/distill.
