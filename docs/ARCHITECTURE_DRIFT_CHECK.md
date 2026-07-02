# Architecture Drift Check

Checklist:

- No database crates or database-backed memory.
- No Python files.
- No LLM calls in `src/core/`.
- Dynamics never read trace files or trace history.
- Labels are inspection-only and never used in settle/write/hebb/decay dynamics.
- Single-threaded deterministic core path.
- No `rand` crate.
- No `HashMap` iteration in dynamics.
- Sparse settle path: `O(T * allocated_rows * K)`, not dense `N*N`.

`tests/drift_check.rs` scans `Cargo.toml` and core sources for the forbidden dependencies and obvious architecture drift markers.

