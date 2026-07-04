# B04 Ingest Axis Map

All B04 ingest events use EG-1 source `test_verified`. Values are clamped by construction to `[-1, 1]` before the existing EG-1 validator sees them.

## cargo-test

Concept: `test_suite`

- `truth_assert`: `+1` when a parsed cargo-test summary has at least one test and zero failures, otherwise `-1`.
- `completion`: `passed / (passed + failed)`.
- `risk`: `failed / (passed + failed)`.
- `confidence_proxy`: same pass ratio as `completion`.

Multiple `test result:` summaries produce multiple deterministic events for the same concept, preserving file order.

## sweep-csv

Concept: `sweep_<hash8>`, where `<hash8>` is the first eight characters of the selected `theta_hash` or `hash` column.

The selected row is the passing row with the highest `recall_margin_095` / `margin`; ties break by higher recall and then lower hash. If no row passes, the first row is emitted as failed evidence.

- `truth_assert`: `+1` for `all_pass=true|1|pass`, otherwise `-1`.
- `completion`: `completion_recall` or `recall`.
- `risk`: `completion_contamination` or `contamination`.
- `confidence_proxy`: `recall_margin_095` or `margin`.

## world-run

Concept: `world_run`

- `truth_assert`: `-1` for `termination=death` / JSON death, otherwise `+1`.
- `completion`: `+1` for `termination=goal` / JSON goal or success, otherwise `-1`.
- `risk`: `+1` for death, otherwise `0`.
- `confidence_proxy`: always `+1` for a parsed non-empty world-run artifact.
