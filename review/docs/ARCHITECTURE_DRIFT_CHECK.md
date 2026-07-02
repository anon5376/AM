# Architecture Drift Check

Checklist:

- No database crates or database-backed memory.
- No vector/RAG store crates.
- No Python files.
- No `rand` in `src/core/`.
- No LLM/Ollama/client/http calls in `src/core/`.
- Dynamics never read trace files or trace history.
- Labels are inspection/resolution metadata only and never allocation state.
- `settle`, `write`, `hebb`, and `decay` must not depend on label metadata.
- Single-threaded deterministic core path: no thread/tokio/rayon/spawn/time reads in `src/core/`.
- No order-sensitive `HashMap` mention in dynamics/output files.
- Semantic quarantine blacklist stays out of learner/world/core paths.
- Sparse settle path: `O(T * allocated_rows * K)`, not dense `N*N`.

`tests/drift_check.rs` scans `Cargo.toml` and Rust sources for forbidden dependencies and obvious architecture drift markers. The lint is intentionally textual and conservative; if it fails, either remove the drift or document and narrow the token use before changing the test.
