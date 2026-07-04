# B02 Workplan

## Mission

Add Track B falsification machinery before any AM-backed synergy feature exists: deterministic corpus, replay/live transport boundary, B0/B1 lanes, B2 unavailable stub, deterministic scoring, frozen metrics, and acceptance tests.

## Files And Order

1. Preserve preflight evidence already produced:
   - `review/b01-complete-head-full-tree.zip`
   - `b01-complete` tag
   - `dashboard-prototype` branch
2. Add B02 source files:
   - `src/beval/mod.rs`
   - `src/beval/corpus.rs`
   - `src/beval/prompt.rs`
   - `src/beval/results.rs`
   - `src/beval/scoring.rs`
   - `src/beval/transport.rs`
3. Wire B02 surfaces:
   - `src/lib.rs`
   - `src/cli/commands.rs`
   - `src/llm/ollama_client.rs`
4. Commit B02 corpus and synthetic replay fixtures:
   - `beval/corpus/*.json`
   - `beval/fixtures/rawlog_v1.md`
   - `beval/fixtures/synthetic_v1/manifest.json`
   - `beval/fixtures/synthetic_v1/<sha256>.txt`
5. Freeze metrics and update drift docs:
   - `beval/METRICS.md`
   - `docs/ARCHITECTURE_DRIFT_CHECK.md`
   - `tests/drift_check.rs`
6. Add B02 tests:
   - `tests/beval_b02.rs`
7. Run:
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo test`
   - demo `am beval` commands for B0, B1, and B2
8. Write:
   - `docs/BUILD_REPORT_B02.md`
   - final full-tree archive in `review/`

## Risks

- Fixture hashes are sensitive to prompt formatting; any prompt edit requires regenerating committed synthetic fixture files.
- The live Ollama transport must stay dormant in tests. B02 records the boundary but does not make model-quality claims.
- The B1 raw-log context intentionally contains stale traps, so answer keys must test current verified state rather than raw-log repetition.

## Deferrals

- B2 context compilation is B03 and remains `LaneUnavailable`.
- Real fixture recording is a user-present step after B02.
- No B03 context compiler, writeback, dashboard, planner, core, world, percept, or sweep changes are in scope.
