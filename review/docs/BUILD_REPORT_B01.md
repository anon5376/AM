# AM001 B01 Build Report

## Workstreams

- Added EG-1 batch event substrate in `src/apply/mod.rs`.
- Added `am apply --snapshot <path> --events <file.jsonl> [--report <out.json>]`.
- Added deterministic `<snapshot>.staging` sidecar with format version `1`.
- Added deterministic `<snapshot>.trace.jsonl` output for event-id provenance joins.
- Added `docs/EVENT_GRAMMAR.md`.
- Extended architecture drift scan for B-track network/LLM/core-leak rules.
- Added B01 integration tests for grammar, apply equivalence, staging lifecycle, sidecar roundtrip/corruption, provenance joins, determinism, and rejection no-mutation behavior.

## Commands Run

```text
$ cargo fmt && cargo test --test apply_b01
running 9 tests
test result: ok. 9 passed; 0 failed
```

```text
$ cargo fmt && cargo clippy -- -D warnings && cargo test
Finished `dev` profile
...
Doc-tests am001
test result: ok. 0 passed; 0 failed
```

Full suite result:

```text
51 tests passed; 0 failed
```

Additional theta evidence commands:

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

## Theta Hashes

Default theta hash used by B01 tests:

```text
6905c5f8beede6bd3dfd9fc403626eafb433488291667610d11f0db54f23520c
```

B01 adds no theta files, no theta fields, and no non-default theta criteria.

Legacy non-default test noted during verification:

```text
link_decay_alive=7c051233480a8d550a4cd577653dd685aba4260fd32869a0c6df651dd65fd3ae
```

## Test Results

B01 tests:

- `grammar_accepts_and_rejects`: pass.
- `apply_matches_step_text`: pass; apply path and step-text path produced byte-identical snapshots and traces.
- `staging_lifecycle`: pass; staged assert contradiction remained staged, same-sign corroboration committed once, and 200 applied non-staged events expired a claim.
- `staging_roundtrip_byte_exact_and_refuses_corruption`: pass.
- `provenance_joinable`: pass for `user`, `test_verified`, and `committed_from_staging`.
- `same_snapshot_and_events_are_byte_deterministic`: pass.
- `fuzz_rejection_never_mutates`: pass with seeded LCG malformed-line generator, no `rand`.
- `cli_apply_writes_report_trace_and_staging`: pass.
- `report_and_trace_serialization_are_stable`: pass.

Existing suite:

- All 42 pre-B01 tests still pass.
- Drift scan with B-track extensions passes.
- No golden trace regeneration was required.

## Core Diff Surface

`git diff -- src/core` is empty.

No core dynamics were changed. No settle, write, Hebb, contradiction, decay, theta, or snapshot schema change was made. Existing `StepTrace.event_id` and `MutationRecord.event_id` were sufficient for provenance joins, so no trace schema metadata field was added.

## Grammar Doc

Present:

```text
docs/EVENT_GRAMMAR.md
```

The doc defines the closed `{"grammar":"EG-1"}` header, envelope fields, supported verbs, rejection reasons, sidecar lifecycle, and anti-laundering rationale.

## Worked Example

Demo command:

```text
tmpdir=$(mktemp -d)
# write EG-1 file containing assert, cue, link, goal_push, goal_pop,
# staged assert commit, staged link commit, rejected llm_claim cue,
# and 200 later applied cue events for expiry
cargo run --quiet -- apply --snapshot "$tmpdir/am.bin" --events "$tmpdir/b01_events.jsonl" --report "$tmpdir/report.json"
cargo run --quiet -- dump --snapshot "$tmpdir/am.bin" --sort act --top 8
```

Expected apply exit:

```text
Error: EG-1 apply completed with rejected events
APPLY_EXIT=1
```

The nonzero exit is correct because event `11` is a rejected `llm_claim` cue.

Report summary:

```json
{
  "summary": {
    "applied": 208,
    "staged": 3,
    "rejected": 1,
    "committed_from_staging": 2,
    "expired": 1,
    "structural_rejections": 0
  }
}
```

Lifecycle report excerpt:

```json
{"id":6,"action":"staged","source":"llm_claim"}
{"id":8,"action":"applied","source":"test_verified"}
{"id":6,"action":"committed_from_staging","source":"llm_claim","committed_by":8}
{"id":9,"action":"staged","source":"llm_claim"}
{"id":10,"action":"applied","source":"test_verified"}
{"id":9,"action":"committed_from_staging","source":"llm_claim","committed_by":10}
{"id":11,"action":"rejected","source":"llm_claim","reason":"CueClaimRejected"}
{"id":12,"action":"staged","source":"llm_claim"}
{"id":212,"action":"applied","source":"user"}
{"id":12,"action":"expired","source":"llm_claim"}
```

Dump excerpt:

```text
rust         [truth_assert +1.00  goal_relevance +0.80]  a=0.00 b=0.05 cert=0.83 age=208
am001        []  a=0.00 b=0.04 cert=0.83 age=207
claim_axis   [truth_assert -0.67]  a=0.00 b=0.05 cert=0.82 age=202  (min-axis cert: truth_assert 0.46)
clock        []  a=0.00 b=0.04 cert=0.83 age=0
```

## Files Changed By B01

- `docs/CODEX_WORKPLAN_B01.md`
- `docs/BUILD_REPORT_B01.md`
- `docs/EVENT_GRAMMAR.md`
- `src/apply/mod.rs`
- `src/cli/commands.rs`
- `src/lib.rs`
- `tests/apply_b01.rs`
- `tests/drift_check.rs`

Pre-existing local dashboard/README changes were preserved and are not B01 work.

## Deviations

- `am apply` writes `<snapshot>.trace.jsonl` even though the CLI line only names `--snapshot`, `--events`, and `--report`. Reason: B01 requires apply-vs-step-text trace equivalence and joinable provenance without modifying core trace schema.
- Multi-axis `llm_claim` asserts are staged per `(concept, axis)`. Reason: corroborating one axis must not launder a sibling axis from the same input event.
- `llm_claim` goal verbs are rejected with `UnimplementedVerb`. Reason: B01 defines no goal corroboration rule and forbids adding new core goal mechanics.

## Known Limitations

- Event ids are guaranteed strictly increasing per EG-1 file, not globally across all future apply files.
- Staging sidecar is bridge-owned JSON, not part of snapshot v4.
- `cue` corroboration is intentionally unsupported; `llm_claim` cues are always rejected.

## Exact Next Tasks

- Add Track B caller-side provenance lookup helper if the next layer needs report/trace joins by `(session_id, event_id)`.
- Add a compact `am apply --dry-run` once there is a consumer that needs preflight without mutation.
- Keep `src/core/` free of staging/provenance policy unless a later audited prompt explicitly changes the core trace schema.
