# AM001 B06 Registered Eval Report

## Scope

- Evaluation session: B06 registered eval plus A18 RECENT ring ratification.
- Evaluation commit: `b7fad9118379bb816307a3e45d4f4d629331df74`.
- Baseline commit before changes: `d5af0b0cb0b46d0cd62d5dbb04935206b53c6b8e`.
- Default theta hash: `6905c5f8beede6bd3dfd9fc403626eafb433488291667610d11f0db54f23520c`.
- Frozen metrics hash: `b26785e36b275bd7457ff704d5b899f12ae7ecd8090b26685d1d68c190e08219  beval/METRICS.md`.

## Required Baseline Evidence

```text
$ git status --short
?? AM-0.0.1.zip

$ git rev-parse HEAD
d5af0b0cb0b46d0cd62d5dbb04935206b53c6b8e

$ cargo test
PASS: 84 tests passed.
```

## CLI Shape Evidence

```text
$ cargo run -- --help || true
PASS: usage is `am <COMMAND>` and includes `beval`.

$ cargo run -- am --help || true
PASS under `|| true`: error was `unrecognized subcommand 'am'`.

$ cargo run -- beval --help || true
PASS: usage is `am beval [OPTIONS] --corpus ... --lane ... --transport ... --fixtures ... --out ...`.
```

## Live Fixture Status

- Live recorded: no.
- Synthetic retained: yes.
- Synthetic manifest: `beval/fixtures/synthetic_v1/manifest.json`.
- Model: none.
- Digest: none.
- Live fixture blocker: live recording was blocked because `env | sort | rg '^(AM_|OLLAMA)' || true` returned no `AM_ENABLE_OLLAMA`, `AM_OLLAMA_MODEL`, or `AM_OLLAMA_DIGEST`, and repo/docs search found no pinned model/digest values to use. No model or digest was chosen or substituted.
- Manifest validation: `fixture_manifest_requires_explicit_synthetic_status` rejects a manifest with omitted `synthetic` status.

## A18 Ratification

- Amendment: A18 appended to `docs/AM_CORE_V0_AMENDMENTS.md`.
- Ring purpose: reporting-only RECENT context aid; not cognition or dynamics input.
- Capacity: fixed at 20 entries via `RECENT_MUTATION_CAUSE_CAPACITY`.
- Overflow: deterministic oldest-entry drop.
- Dedup rule: ring storage does not dedup; RECENT context display dedups only by displayed cause text with counts, preserving distinct event ids in the ring.
- Event-id join: ring entries retain `event_id` where available; no missing-id join fallback is invented.
- Allowed readers: snapshot serialization/deserialization, context compilation, dashboard/provenance/reporting, and tests.
- Guard test: `dynamics_sources_do_not_read_recent_ring`.

## A18 Verification

```text
$ cargo fmt && cargo test --test a18_recent_ring -- --nocapture
PASS: 3 tests passed.

$ cargo test --test beval_b03 -- --nocapture
PASS: 6 tests passed.

$ cargo fmt && cargo clippy -- -D warnings && cargo test
PASS: 88 tests passed.
```

## B06 Commands

```text
$ cargo run --quiet -- init --snapshot artifacts/b06/b06_b2_snapshot.bin
PASS

$ cargo run --quiet -- step-text --snapshot artifacts/b06/b06_b2_snapshot.bin ...
PASS: deterministic B2 snapshot seeded for compiled-context lane.

$ cargo run --quiet -- compile-context --snapshot artifacts/b06/b06_b2_snapshot.bin --budget 1200 --out artifacts/b06/b2_context_1200.txt
compile-context out=artifacts/b06/b2_context_1200.txt tokens=535

$ cargo run --quiet -- beval --corpus beval/corpus --lane b0 --transport replay --fixtures beval/fixtures/synthetic_v1 --out artifacts/b06/b06_b0.json
beval lane=b0 transport=replay evaluated=24 skipped=5 out=artifacts/b06/b06_b0.json

$ cargo run --quiet -- beval --corpus beval/corpus --lane b1 --transport replay --fixtures beval/fixtures/synthetic_v1 --out artifacts/b06/b06_b1.json
beval lane=b1 transport=replay evaluated=24 skipped=5 out=artifacts/b06/b06_b1.json

$ cargo run --quiet -- beval --corpus beval/corpus --lane b2 --transport replay --fixtures beval/fixtures/synthetic_v1 --snapshot artifacts/b06/b06_b2_snapshot.bin --out artifacts/b06/b06_b2.json
beval lane=b2 transport=replay evaluated=29 skipped=0 out=artifacts/b06/b06_b2.json
```

## B06 Rows

| Lane | Role | Fixture | Evaluated | Skipped | stage_accuracy | stale_claim | contradiction_handling | test_grounded | drift | Stale claim rate | Drift rate | Total context tokens | Mean context tokens |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B0 | Diagnostic only | synthetic | 24 | 5 | 8/8 | 5/5 | 0/0 | 5/5 | 6/6 | 0.0 | 0.0 | 0 | 0.0 |
| B1 | Raw-log text baseline | synthetic | 24 | 5 | 8/8 | 5/5 | 0/0 | 5/5 | 6/6 | 0.0 | 0.0 | 7392 | 308.0 |
| B2 | Compiled AM context | synthetic | 29 | 0 | 8/8 | 5/5 | 5/5 | 5/5 | 6/6 | 0.0 | 0.0 | 15515 | 535.0 |

## Category Breakdown

- stage_accuracy: 8 tasks.
- stale_claim traps: 5 tasks.
- contradiction_handling: 5 tasks.
- test_grounded: 5 tasks.
- drift: 6 tasks.

## Token Gate

- B1 total context tokens: 7392.
- B1 mean context tokens: 308.0.
- B2 total context tokens: 15515.
- B2 mean context tokens: 535.0.
- B1/B2 token ratio: 0.47644215275539803.
- Required B1/B2 token ratio for B2 to use at least 5x fewer tokens: >= 5.0.

## Deterministic Matcher Summary

- Transport: replay.
- Fixture manifest version: `synthetic_v1`.
- B0 deterministic matcher failures: none.
- B1 deterministic matcher failures: none.
- B2 deterministic matcher failures: none.

## B06 Verdict

B06 FAIL: parameter memory is not yet beating text memory on the frozen 29-task corpus. Next session is diagnosis, not features.

## Exact Failure List

- B2 token cost failed the registered gate: B2 total context tokens = 15515, B1 total context tokens = 7392, B1/B2 token ratio = 0.47644215275539803, required >= 5.0.
- B2 did not beat B1 under B06 acceptance because the token-reduction gate failed.

## Artifact Hashes

```text
8fb55af30ffec8a9a2c98f039507e9933d8ee149c7f457b7a32852576699869a  artifacts/b06/b06_b0.json
d2bdc5a73b1f8d980b1e5e44a3b01ec191c69580114280a4043e52bb84f13f75  artifacts/b06/b06_b1.json
52f8056b8e09d847e5c40ec2db4691cb324854e71bb47e72bd337c7324f971b5  artifacts/b06/b06_b2.json
fe0cf3384e9cc93e6427ecf6b43654b26cdddbf3e7fe3d3c6d984f9b40db6c42  artifacts/b06/b2_context_1200.txt
e1bd5d707356b2e0d6d9e2fd1da88b7e9f0aec9e0a311cba0a3cc0d478e2cb8f  artifacts/b06/b06_b2_snapshot.bin
```

## Review Archive

```text
$ scripts/make_review_archive.sh review.zip artifacts/review_archive_manifest.txt
archive=review.zip
manifest=artifacts/review_archive_manifest.txt
```

- Archive path: `review.zip`.
- Manifest path: `artifacts/review_archive_manifest.txt`.
- Archive includes `artifacts/`: yes.
