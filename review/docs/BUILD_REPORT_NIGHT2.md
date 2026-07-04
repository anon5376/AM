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
- Added snapshot-resident `recent_mutation_causes` inspection state and bumped the core snapshot format to v5 so `RECENT` is the last 20 mutation causes, deduplicated and counted, without reading trace logs.
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
30da77c3438f4770c88f86f262e22d118aa779c6f8d9471db24f22ea383f169e  -
```

### Files Changed

- `src/beval/compile.rs`
- `src/beval/mod.rs`
- `src/beval/prompt.rs`
- `src/cli/commands.rs`
- `src/core/snapshot.rs`
- `src/core/state.rs`
- `src/core/step.rs`
- `tests/beval_b02.rs`
- `tests/beval_b03.rs`
- `tests/golden/b03_context_1200.txt`
- `tests/golden/b03_context_400.txt`
- `beval/fixtures/synthetic_v1/*.txt` (78 frozen replay prompt fixtures total)
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

- The frozen B02 corpus/matchers were not edited. This means `stale_b2_ready` still tests the B02-era stale claim by design while B03 proves the new B2 lane separately.

### Known Limitations

- Synthetic replay fixtures prove deterministic prompt hashing, context budgeting, replay, and scoring. They are not model-quality evidence.

### Next

B04 ingest/distill.

## B04 — Ingest and Distill Writeback

### Workstream

- Added B04 implementation under `src/beval/ingest.rs`, with `src/beval/distill.rs` as the named distill entrypoint.
- Added `am ingest --kind cargo-test|sweep-csv|world-run --input <f> --session <id> --events-out <f.jsonl>`.
- Added `am distill --input <llm_output.txt> --session <id> --events-out <f.jsonl>`.
- Ingest outputs `test_verified` EG-1 events only, using the pinned B04 concepts `test_suite`, `sweep_<hash8>`, and `world_run` with values constrained to `[-1, 1]`.
- Added `docs/INGEST_AXIS_MAP.md` for cargo-test, sweep-csv, and world-run mappings onto existing axes.
- Distill extracts exactly one fenced `eg1` JSONL block without a header, forces `session_id` and `source=llm_claim`, validates the generated EG-1 through the existing B01 parser, and writes nothing on rejection.
- Added end-to-end coverage proving a distilled `llm_claim` assertion stages first and commits only after a `test_verified` cargo-test ingest event corroborates the same concept/axis sign.

### Commands

```text
$ cargo test --test ingest_b04 -- --nocapture
running 5 tests
test ingest_and_distill_outputs_are_deterministic ... ok
test ingest_rejects_malformed_inputs_without_partial_output ... ok
test ingest_all_kinds_emit_valid_test_verified_events ... ok
test distill_extracts_one_fence_forces_source_and_rejects_bad_inputs ... ok
test distilled_claim_stages_and_ingested_test_verified_event_commits_it ... ok
test result: ok. 5 passed; 0 failed
```

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
74 tests passed; 0 failed
```

### Demo

```text
$ am distill --input <llm.txt> --session s_demo --events-out <claim.jsonl>
distill events_out=<claim.jsonl> bytes=159

$ am ingest --kind cargo-test --input <cargo.txt> --session s_demo --events-out <verified.jsonl>
ingest kind=cargo-test events_out=<verified.jsonl> bytes=237
```

Distilled claim:

```json
{"grammar":"EG-1"}
{"args":{"axes":[{"axis":"truth_assert","value":1.0}],"concept":"test_suite"},"id":1,"session_id":"s_demo","source":"llm_claim","verb":"assert"}
```

Verified ingest:

```json
{"grammar":"EG-1"}
{"args":{"axes":[{"axis":"truth_assert","value":1.0},{"axis":"completion","value":1.0},{"axis":"risk","value":0.0},{"axis":"confidence_proxy","value":1.0}],"concept":"test_suite"},"id":1,"session_id":"s_demo","source":"test_verified","verb":"assert"}
```

Apply summaries:

```json
{"summary":{"applied":0,"staged":1,"rejected":0,"committed_from_staging":0,"expired":0,"structural_rejections":0},"events":[{"id":1,"action":"staged","source":"llm_claim"}]}
{"summary":{"applied":1,"staged":0,"rejected":0,"committed_from_staging":1,"expired":0,"structural_rejections":0},"events":[{"id":1,"action":"applied","source":"test_verified"},{"id":1,"action":"committed_from_staging","source":"llm_claim","committed_by":1}]}
```

### Files Changed

- `src/beval/distill.rs`
- `src/beval/ingest.rs`
- `src/lib.rs`
- `src/cli/commands.rs`
- `tests/ingest_b04.rs`
- `tests/fixtures/b04/*`
- `docs/INGEST_AXIS_MAP.md`
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

- Cargo-test, sweep, and world-run parsers intentionally emit compact verification summaries, not raw artifact text, individual test names, or world logs. This preserves the Track B no-raw-text boundary.
- The sweep parser handles the project sweep CSV shape without adding a CSV dependency; quoted CSV fields are not supported because current sweep artifacts do not require them.

### Known Limitations

- Distill validates and stages proposed EG-1 events; it does not infer events from arbitrary prose.

### Next

B05 static dashboard.

## B05 — Static Dashboard

### Workstream

- Added `src/dashboard/mod.rs`.
- Added `am dashboard --snapshot <p> [--beval <results.json>...] [--trace <trace.jsonl>...] [--report <apply.json>...] --out <dash.html>`.
- Dashboard output is a single self-contained HTML file with embedded JSON, inline CSS, and vanilla inline JS. It has no server, no network URLs, no external script/link tags, and no dependency on the quarantined prototype branch.
- Panels cover compiled context at budget 2000, active rows, strong links, open contradictions, staging queue with per-entry age and downloadable corroboration EG-1 template, B-eval results with token costs/category accuracy, and a provenance browser for the last 200 trace events joined with apply-report actions.
- Added deterministic golden coverage, required-anchor greps, and an embedded JSON smoke test.

### Commands

```text
$ cargo test --test dashboard_b05 -- --nocapture
running 2 tests
test dashboard_embedded_json_smoke ... ok
test dashboard_cli_matches_golden_and_is_self_contained ... ok
test result: ok. 2 passed; 0 failed
```

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
76 tests passed; 0 failed
```

### Demo

```text
$ am dashboard --snapshot <dash.bin> --beval tests/golden/b05_beval.json --trace <dash.bin.trace.jsonl> --report <report.json> --out tests/golden/b05_dashboard.html
dashboard out=tests/golden/b05_dashboard.html bytes=9707
```

### Hashes

```text
$ shasum -a 256 tests/golden/b05_dashboard.html tests/golden/b05_beval.json
2ffb3c3f5bfdbc3f863e25affe037d75e104d8b46a9c43f3ce3db3aef2d52eac  tests/golden/b05_dashboard.html
c82201cd65da4303b368d50ce17b26c49f6acf162b6cc8f65a2146b62fb43e47  tests/golden/b05_beval.json
```

### Files Changed

- `src/dashboard/mod.rs`
- `src/lib.rs`
- `src/cli/commands.rs`
- `tests/dashboard_b05.rs`
- `tests/golden/b05_dashboard.html`
- `tests/golden/b05_beval.json`
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

- B05 implements the static inspection dashboard requested in the Night2 block. It does not implement the broader future B05 roadmap lane comparison/apply-reject UI; that remains outside this block and would require a separate prompt.

### Known Limitations

- Dashboard HTML is read-only. It intentionally cannot mutate snapshots, staging, or EG-1 files.

### Next

Extras: `am apply --dry-run`, `am provenance --event <id>`, `docs/TRACK_B_README.md`, final archive/review zip.

## Extras — Dry Run, Provenance, README

### Workstream

- Added `am apply --dry-run`, using the normal EG-1 parser and apply simulation while skipping snapshot, staging, and trace writes.
- Added `am provenance --event <id> --snapshot <p>` and `am provenance --event <id> --trace <trace.jsonl>`.
- Added `docs/TRACK_B_README.md` with the current local Track B command flow and boundaries.

### Commands

```text
$ cargo test --test track_b_extras -- --nocapture
running 2 tests
test apply_dry_run_reports_without_writing_state_sidecars_or_trace ... ok
test provenance_summarizes_apply_trace_by_event_id ... ok
test result: ok. 2 passed; 0 failed
```

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
78 tests passed; 0 failed
```

### Demo

```text
$ am apply --dry-run --snapshot <am.bin> --events <events.jsonl> --report <dry.json>
dry_snapshot_exists=no dry_trace_exists=no
{"applied":1,"staged":0,"rejected":0,"committed_from_staging":0,"expired":0,"structural_rejections":0}
```

```text
$ am provenance --snapshot <am.bin> --event 1
event 1
trace_file <am.bin.trace.jsonl>
traces 1
ticks 1
mutations 55
opened_contradictions 0
closed_contradictions 0
causes:
  Allocate x53
  ActivationDecay x1
  RowGeneration x1
targets:
  M x1
  V x48
  A x1
  B x1
  U x1
  Generation x1
  Label x1
  Allocation x1
```

### Files Changed

- `src/provenance/mod.rs`
- `src/lib.rs`
- `src/cli/commands.rs`
- `tests/track_b_extras.rs`
- `docs/TRACK_B_README.md`
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

- `am provenance` requires `--snapshot` or `--trace` because there is no global trace database and adding one would violate the local no-database boundary.

### Known Limitations

- Provenance reports trace-level mutation summaries by `event_id`; it does not reconstruct the original EG-1 envelope if the apply report file is not supplied.

### Next

Create final review artifacts and push.

## Final Audit Gate

### Commands

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
clippy: ok
tests: 84 passed; 0 failed
doc-tests: 0 passed; 0 failed
```

Focused reruns after the audit corrections:

```text
$ cargo test --test beval_b03 -- --nocapture
test result: ok. 6 passed; 0 failed

$ cargo test --test ingest_b04 -- --nocapture
test result: ok. 5 passed; 0 failed

$ cargo test --test dashboard_b05 -- --nocapture
test result: ok. 2 passed; 0 failed
```

### Final Corrections

- B03 `RECENT` now uses snapshot-resident mutation-cause history, not `recent_writes`; snapshot format is v5.
- B04 ingest/distill now has committed fixtures, malformed-input tests, deterministic-output tests, and `docs/INGEST_AXIS_MAP.md`.
- B05 dashboard now includes compiled context, staging queue with corroboration downloads, token-cost B-eval rows, and trace/report provenance joins.

### Known Limitations

- The Track B dashboard is still read-only inspection. It intentionally does not mutate snapshots or call a live LLM.
