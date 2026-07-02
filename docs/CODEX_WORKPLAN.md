# AM001 Codex Workplan

Files inspected:

- `docs/AM_CORE_V0.md`
- `docs/AM_CORE_V0_AMENDMENTS.md`
- attached build prompt

Workstreams:

- Architecture Auditor: keep the implementation aligned to the non-negotiable laws, module layout, and no-database/no-Python/no-rand constraints.
- Core Math: implement `Theta`, axes, state arrays, P1 through P7, sparse links, contradiction pressure, decay, merge, and GC.
- Determinism/Trace: canonical JSONL trace, deterministic iteration, state and trace hashes, mutation records at mutation sites.
- Storage: bincode snapshots including recent write history and contradictions.
- CLI/Inspectability: `init`, `dump`, `step`, `step-text`, `run`, `repl`, `bench-step`, diff formatting.
- Kill-Test: determinism, completion, reinforcement, contradiction, forgetting, diff integrity, snapshot roundtrip, parser, CLI smoke.
- Ollama-Seam: stub parser/client gated outside the core.
- Integration Auditor: drift check doc and dependency scan test.

Order:

1. Source doc and amendments.
2. Core data model and mutation logging.
3. Step phases.
4. Storage, parser, CLI.
5. Tests and tuned theta files only if needed.
6. Real demo transcript and build report.

Risks:

- The completion threshold may require calibrated theta while preserving Hebbian completion.
- Diff integrity is strict, so every mutation path must flow through logging helpers.
- GC and merge can accidentally create order dependence unless candidate rows are sorted.

Deferrals:

- Full TUI watch.
- Real Ollama renderer.
- Daemon loop process.
- Relational operators.
- Episodic fast-weight matrix.
- Neural training.

