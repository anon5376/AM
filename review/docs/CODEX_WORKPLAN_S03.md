# CODEX Workplan S03

## Scope

Harden the AM substrate after S02 by removing the audited allocation/W-link debts:

- Allocation state becomes an explicit parameter store, not inferred from labels.
- W/free/merge mutations are logged at mutation sites and preserve final diff integrity.
- Stale `RowRef` and stale W-link paths are covered by tests.
- Drift checks are strengthened for allocation, deterministic core, dormant Ollama seam, and semantic quarantine.
- Reports stay honest about the 729-point theta grid: no impossibility claim unless wider evidence exists.

## Files and Order

1. `src/core/state.rs`
   - Add explicit `allocated: Vec<bool>` snapshot state.
   - Add logged allocation-state mutation helper.
   - Expose coalesced mutation logging for W mutation sites.
   - Move free-time label/goal/write metadata cleanup behind state helpers so dynamics do not use label presence as allocation state.

2. `src/core/resolve.rs`
   - Set allocation state on row allocation.
   - Keep generation bump and label metadata deterministic.

3. `src/core/hebb.rs`
   - Apply W decay only when the decay delta reaches `eps_log`.
   - Log W changes through the coalescing logger exactly once per final changed cell.

4. `src/core/decay.rs`
   - Keep `Link.target: usize` and prove stale links are removed.
   - Remove and log outgoing and inbound W links on free.
   - Redirect/remove and log W links on merge.
   - Ensure free/reuse cannot leave inbound links to a freed row.

5. `tests/diff_integrity.rs`
   - Add allocation state to diff coverage.
   - Add scenarios for ordinary events, contradiction open/close, GC/free outgoing and inbound links, slot reuse, merge inbound/outgoing links, long empty-tick sub-eps W decay, and snapshot roundtrip after free/merge/reuse.

6. `tests/stale_w_links.rs`
   - Prove inbound/outgoing links are removed before slot reuse and no stale W edge reaches the reused row.

7. `tests/drift_check.rs` and `docs/ARCHITECTURE_DRIFT_CHECK.md`
   - Expand drift checks for forbidden stores/deps, dormant Ollama seam, no runtime randomness/threading/time in core, label-free dynamics, and semantic quarantine.

8. `docs/BUILD_REPORT_S03.md`
   - Capture workstreams, commands, full test/demo outputs, theta hashes, deviations, limits, and exact next tasks.

## Verification Commands

- `cargo fmt --check`
- `cargo clippy`
- `cargo test -- --nocapture`
- AM001 manual demo commands
- stale row/W-link demo command
- `cargo run -- axes rust`
- `cargo run -- bench-step --events 1000`

## Risks

- Coalesced mutation logging must not hide the mutation-site cause while still producing one final record per changed cell.
- Free and merge can touch the same W cell as Hebbian update in a single tick; tests must enforce final-state trace integrity.
- Snapshot format is already v2 from S02. Adding `allocated` changes the state layout, so S03 will bump the snapshot format again and keep clear mismatch refusal.

## Deferrals

- No new theta search beyond the requested S02 grid unless tests force a constant-only fix.
- No phase formula changes.
- No RAG, database memory, raw chat memory, runtime LLM calls, threading, or core randomness.
