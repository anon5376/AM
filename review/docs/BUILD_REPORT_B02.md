# AM001 B02 Build Report

## Workstreams

- Preserved true pre-session state as a full-tree `git archive HEAD`:
  - `review/b01-complete-head-full-tree.zip`
- Tagged B01 completion:
  - `b01-complete`
- Quarantined the unaudited dashboard prototype to a separate branch:
  - branch: `dashboard-prototype`
  - head: `3d529cf068309bcd8eec1fca50d131f809a0ad17`
- Added B02 falsification harness under `src/beval/`.
- Added `am beval --corpus <dir> --lane b0|b1|b2 --transport replay|live --fixtures <dir> [--record] --out <results.json>`.
- Added deterministic replay transport keyed by `sha256(prompt)`.
- Added dormant live Ollama transport in `src/llm/ollama_client.rs`; it requires `AM_ENABLE_OLLAMA=1`, pinned `AM_OLLAMA_MODEL`, pinned `AM_OLLAMA_DIGEST`, temperature `0`, and optional seed. It was not run.
- Added corpus, raw-log fixture, synthetic replay fixtures, and frozen metrics.
- Added deterministic scoring: envelope extraction plus exact, regex, must-contain-all, and must-contain-none matchers.
- Added B2 `LaneUnavailable` behavior for B02; no context compiler was implemented.
- Updated drift scan for B02: at most one scoped HTTP client dependency is permitted, and HTTP client imports are confined to `src/llm/`. This implementation added no HTTP dependency and uses only `std::net` inside `src/llm/ollama_client.rs`.

## Corpus

Committed corpus path:

```text
beval/corpus/
```

Task count:

```text
29
```

Category counts:

```text
stage_accuracy=8
stale_claim=5
contradiction_handling=5
test_grounded=5
drift=6 tasks / 3 repeated-question groups
```

Contradiction tasks are marked `requires_lane: "b2"` and are skipped without penalty in B0/B1. B2 is unavailable in B02 and skips all tasks as `LaneUnavailable`.

## Synthetic Fixture Disclosure

Committed replay fixture path:

```text
beval/fixtures/synthetic_v1/
```

Manifest:

```json
{
  "version": "synthetic_v1",
  "synthetic": true,
  "fixture_format": "sha256-prompt-v1"
}
```

The fixtures are hand-written synthetic responses. They prove transport, prompt hashing, deterministic replay, and scoring only. They are not model-quality evidence and no real-model claim is made from them.

No live LLM ran during tests or demos.

## Metrics

Frozen metrics file:

```text
beval/METRICS.md
```

SHA-256:

```text
b26785e36b275bd7457ff704d5b899f12ae7ecd8090b26685d1d68c190e08219
```

B06 gate recorded in metrics:

```text
B2 must beat B1 with at least 5x fewer context tokens, or the finding is diagnosis.
```

## Commands Run

Preflight on clean main after dashboard quarantine:

```text
$ cargo fmt --check && cargo clippy -- -D warnings && cargo test
51 tests passed; 0 failed
```

Focused B02 tests:

```text
$ cargo fmt && cargo test --test beval_b02 -- --nocapture
running 9 tests
test corpus_schema_validates ... ok
test scorer_matchers_deterministic ... ok
test replay_double_run_byte_identical ... ok
test b2_lane_unavailable_skips_without_zero_fill ... ok
test token_truncation_deterministic_and_budgeted ... ok
test stale_trap_scoring_correct ... ok
test missing_fixture_errors_cleanly ... ok
test live_transport_never_used_by_tests ... ok
test apply_rejection_exit_prints_one_clean_error_line ... ok
test result: ok. 9 passed; 0 failed
```

Focused drift scan:

```text
$ cargo test --test drift_check -- --nocapture
test architecture_drift_scan_rejects_forbidden_dependencies_and_python ... ok
test result: ok. 1 passed; 0 failed
```

Full final gate:

```text
$ cargo fmt --check && cargo clippy -- -D warnings && cargo test
running 20 tests in src/lib.rs ... ok
running 0 tests in src/main.rs ... ok
running 9 tests in tests/apply_b01.rs ... ok
running 9 tests in tests/beval_b02.rs ... ok
running all existing integration tests ... ok
Doc-tests am001 ... ok
69 tests passed; 0 failed
```

Theta confirmation:

```text
$ cargo test --test completion -- --nocapture
theta_hash=6905c5f8beede6bd3dfd9fc403626eafb433488291667610d11f0db54f23520c
test partial_cue_completes_linked_assemblies_without_foreign_spill ... ok
```

```text
$ cargo test --test link_decay_alive -- --nocapture
theta_hash=7c051233480a8d550a4cd577653dd685aba4260fd32869a0c6df651dd65fd3ae
test nonzero_link_decay_decreases_and_prunes_with_cause_records ... ok
```

Metrics hash:

```text
$ shasum -a 256 beval/METRICS.md
b26785e36b275bd7457ff704d5b899f12ae7ecd8090b26685d1d68c190e08219  beval/METRICS.md
```

## Demo Commands

B0 replay:

```text
$ cargo run --quiet -- beval --corpus beval/corpus --lane b0 --transport replay --fixtures beval/fixtures/synthetic_v1 --out "$tmpdir/b0.json"
beval lane=b0 transport=replay evaluated=24 skipped=5 out=.../b0.json
stage_accuracy=8/8
stale_claim=5/5
contradiction_handling=0/0
test_grounded=5/5
drift=6/6
stale_claim_rate=0.0
answer_drift=0.0
mean_context_tokens=0.0
```

B1 replay:

```text
$ cargo run --quiet -- beval --corpus beval/corpus --lane b1 --transport replay --fixtures beval/fixtures/synthetic_v1 --out "$tmpdir/b1.json"
beval lane=b1 transport=replay evaluated=24 skipped=5 out=.../b1.json
stage_accuracy=8/8
stale_claim=5/5
contradiction_handling=0/0
test_grounded=5/5
drift=6/6
stale_claim_rate=0.0
answer_drift=0.0
mean_context_tokens=308.0
```

B2 replay:

```text
$ cargo run --quiet -- beval --corpus beval/corpus --lane b2 --transport replay --fixtures beval/fixtures/synthetic_v1 --out "$tmpdir/b2.json"
beval lane=b2 transport=replay evaluated=0 skipped=29 out=.../b2.json
first_skip=LaneUnavailable
transport.synthetic=true
fixture_manifest_version=synthetic_v1
```

## Test Results

B02-specific tests:

- `corpus_schema_validates`: pass; 29 tasks, required category minima met.
- `scorer_matchers_deterministic`: pass; exact, regex, must-contain-all, must-contain-none, and missing envelope paths tested.
- `replay_double_run_byte_identical`: pass; B0 and B1 full replay results are byte-identical across repeated runs.
- `b2_lane_unavailable_skips_without_zero_fill`: pass; B2 skips all tasks with `LaneUnavailable`.
- `token_truncation_deterministic_and_budgeted`: pass; token counter is `ceil(chars/4)` and recency truncation keeps the tail.
- `stale_trap_scoring_correct`: pass; stale parroting is counted and a qualified correction is clean.
- `missing_fixture_errors_cleanly`: pass; missing replay fixtures report hash and prompt preview.
- `live_transport_never_used_by_tests`: pass; tests do not import or enable live transport.
- `apply_rejection_exit_prints_one_clean_error_line`: pass; normal rejection exits as one clean error line without a backtrace.

Existing tests:

- All existing 51 pre-B02 tests still pass.
- Full suite now has 69 tests.

## Files Changed By B02

- `AM_B02_PROMPT.md`
- `beval/METRICS.md`
- `beval/corpus/*.json`
- `beval/fixtures/rawlog_v1.md`
- `beval/fixtures/synthetic_v1/manifest.json`
- `beval/fixtures/synthetic_v1/*.txt`
- `docs/ARCHITECTURE_DRIFT_CHECK.md`
- `docs/BUILD_REPORT_B02.md`
- `docs/CODEX_WORKPLAN_B02.md`
- `src/beval/*.rs`
- `src/cli/commands.rs`
- `src/lib.rs`
- `src/llm/ollama_client.rs`
- `tests/beval_b02.rs`
- `tests/drift_check.rs`

## Scope Guard

Untouched:

- `src/core/`
- `src/world/`
- `src/percept/`
- `src/eval/`

No new dependencies were added. No network crate was added. No live LLM was called. No B03 context compiler, writeback, dashboard, planner, core dynamics, world behavior, perception bridge, or sweep change was made.

## Deviations

- The prompt permits one scoped HTTP client dependency, but B02 did not add one. Reason: the dormant live seam can use `std::net` inside `src/llm/`, which keeps the dependency surface smaller while preserving the live boundary for a future user-present recording run.
- `AM_B02_PROMPT.md` was committed at repo root because the active objective refers to that filename directly and it was otherwise only present as an attachment.

## Known Limitations

- B2 is intentionally unavailable until B03; B02 records `LaneUnavailable` and skips rather than zero-filling.
- Synthetic replay fixtures do not measure model quality.
- Live fixture recording requires user-present environment configuration and was not run.
- The regex matcher is a deterministic minimal matcher for B02 keys and tests, not a general regex engine.

## Exact Next Tasks

1. B03: implement snapshot-to-context compiler with stable ordering and a hard 1,200-token budget.
2. Record real B0/B1 fixtures in a user-present live run, replacing only the replay fixture set and marking manifest `synthetic=false`.
3. Add B04 writeback/test-ingestion only after B03 context compiler is audited.
