# AM001 S02 Workplan

Files inspected:

- `docs/AM_CORE_V0.md`
- `docs/AM_CORE_V0_AMENDMENTS.md`
- `AM_ROADMAP_v2_1.md`
- `src/core/state.rs`
- `src/core/event.rs`
- `src/core/resolve.rs`
- `src/core/decay.rs`
- `src/core/inspect.rs`
- `src/core/theta.rs`
- `src/core/trace.rs`
- `src/core/snapshot.rs`
- `src/storage/snapshot_file.rs`
- `src/cli/commands.rs`

Source placement:

- Copied `AM_ROADMAP_v2_1.md` to `docs/AM_ROADMAP_V2.md`.

Workstreams:

- Architecture Auditor: enforce L1-L24, especially snapshot versioning, row generations, dormant Ollama seam, and no semantic leaks in learner/core code.
- Core State: add `format_version`, `generation`, `RowRef`, stale-id rejection, snapshot refusal, trace targets/causes.
- Eval/Sweep: move kill-test criteria into `src/eval/criteria.rs`, add `am sweep`, CSV output, and best-point reporting.
- Inspectability: add min per-axis certainty in `dump` and full `am axes <label>`.
- CLI/Performance: make `bench-step` print mean/max latency.
- Test: add sweep smoke, stale rowref, snapshot format, axes/dump coverage, and keep all existing kill tests green.
- Reporting: run fmt/clippy/test/demo and write `docs/BUILD_REPORT_S02.md`.

Order:

1. Place roadmap and write this workplan.
2. Add snapshot version and row-generation plumbing while preserving existing label workflows.
3. Extract eval criteria and wire the sweep command.
4. Add inspection and benchmark output.
5. Update/add tests.
6. Run required commands and write report.

Risks:

- A uniform theta may not exist in the requested grid without formula changes; if so, keep completion theta and report best points with data.
- Row generation touches all stored ids (`goals`, aliases, contradictions, recent writes) and can break determinism if iteration order is not preserved.
- Snapshot versioning is intentionally incompatible with v1 snapshots; tests must prove clear refusal.
- Sweep runtime can be large; the full grid is 729 points and each point runs six criteria.

Deferrals:

- No world harness, semantic observation loop, typed relations, schemas, or planning in S02.
- No live Ollama implementation.
- No binary snapshot migrator from v1 to v2; mismatched snapshots are refused.

