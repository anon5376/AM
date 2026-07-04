# B01 Workplan

## Mission

Add a validated, provenance-aware batch event write path for Track B without changing AM core dynamics. The path is `am apply`, EG-1 JSONL parsing, deterministic staging sidecar behavior, reports, and tests.

## Files And Order

1. Grammar and staging substrate:
   - Add `src/apply/` with closed EG-1 serde enums, batch parser, validation, staging sidecar wire structs, deterministic report output, and apply execution.
   - Export `apply` from `src/lib.rs`.
   - Keep `src/core/` untouched unless trace metadata proves unavoidable; expected core diff is none because `StepTrace` and `MutationRecord` already carry `event_id`.

2. CLI:
   - Add `am apply --snapshot <path> --events <file.jsonl> [--report <out.json>]`.
   - Parse the full EG-1 file before loading or mutating the snapshot.
   - Save `<snapshot>.staging` beside the snapshot.
   - Save `<snapshot>.trace.jsonl` for joinable provenance inspection.

3. Documentation:
   - Add `docs/EVENT_GRAMMAR.md` documenting the closed EG-1 header, envelopes, verb arguments, staging lifecycle, and anti-laundering rationale.

4. Tests:
   - Add table-driven grammar acceptance/rejection tests.
   - Add apply-vs-step-text equivalence, staging lifecycle, byte-exact sidecar roundtrip, corruption refusal, provenance joinability, determinism, and rejection-never-mutates coverage.
   - Extend `tests/drift_check.rs` with B-track rules for network crates, LLM/client imports, raw-text persistence into core state, and no `src/core/` B01 expansion.

5. Report:
   - Run `cargo fmt && cargo clippy -- -D warnings && cargo test`.
   - Run a worked B01 demo exercising all verbs and lifecycle outcomes.
   - Write `docs/BUILD_REPORT_B01.md` with commands, test count/result, theta hashes, core diff surface, example report JSON, dump excerpt, deviations, limitations, and exact next tasks.

## Risks

- Existing local dashboard edits touch CLI files; B01 must preserve them and avoid accidentally staging unrelated dashboard files unless explicitly requested.
- `llm_claim` must remain inert unless corroborated; cue claims are rejected outright.
- Parse-time file structure errors must reject the whole file before state mutation.
- Per-event semantic rejections must not mutate for that event, while valid earlier/later events still run in deterministic file order.
- Sidecar corruption or version mismatch must refuse loudly, never reset.

## Deferrals

- No network calls.
- No LLM runtime integration.
- No new core dynamics or theta fields.
- No new core goal operations; EG-1 goal verbs only delegate to the existing goal mechanism.
- No raw text persistence into AM state.
