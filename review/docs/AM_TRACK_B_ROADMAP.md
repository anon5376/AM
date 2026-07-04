# AM TRACK B — LLM ⇄ AM001 integration roadmap (binding)
Status: A-track (W-tiers, schemas, planner, P10 transfer proof) FROZEN at the S07 tag after S07 audit passes. Freeze is clean and resumable; the transfer thesis remains unproven and unclaimed. B-track proves a different thesis:

**Thesis B:** a small, deterministic, inspectable parameter-memory state (AM001) makes an LLM measurably better at long-horizon project work than the same LLM with generous raw-log context — at a fraction of the context tokens — and forces all memory writes through validated, cause-tagged mutations.

Constitutional position: this is the "language binding (I/O only, never the brain)" the roadmap deferred, pulled forward deliberately. The LLM stays at the membrane: it proposes events and consumes compiled context. It never settles, writes, plans inside, or reads traces as dynamics. All A-track laws (L1–L24, amendments A1–A17) remain in force on the AM side. Determinism boundary: everything AM-side stays byte-deterministic; the LLM is outside the boundary; the eval harness pins everything pinnable (local model, version, temperature 0) and scores deterministically.

## Event grammar (closed, versioned: EG-1)
Verbs: `assert concept axis=value …` | `cue concept strength` | `link a b weight` | `goal_push …` / `goal_pop` (only if the core's A1 goal mechanism already exposes ops; otherwise the verbs are reserved and rejected with `UnimplementedVerb` — no core additions for them).
Envelope on every event: `id`, `session_id`, `source ∈ {user, test_verified, llm_claim}`, `ts_tick`. Anything outside the grammar is rejected with a machine-readable reason. Grammar changes require a version bump and a migration note; no silent extensions.

## Provenance & staging (the anti-laundering law)
`user` and `test_verified` events apply immediately through the existing step path (normal cause-tagged mutations). `llm_claim` events NEVER apply directly: they enter a bridge-owned staging sidecar (deterministic serialization, own roundtrip test; B01 itself did not change the core snapshot schema). A staged claim commits iff a later `test_verified` or `user` event asserts the same (concept, axis) with the same sign; it expires after 200 applied events otherwise. Stage, commit, and expiry are all logged; committed events join the normal trace with their provenance retrievable by event-id join. Rationale, pinned: the core's delta rule reinforces repetition — unstaged LLM claims would let the model inflate its own confabulations into high-certainty parameter memory.

## Baselines (pre-registered, falsification-grade)
B0 = bare LLM. B1 = LLM + recency-windowed raw session logs, budget 8,000 tokens. B2 = LLM + AM-compiled context, budget ≤1,200 tokens. Same pinned model, same temperature 0, same task set. Success criterion for Thesis B: B2 ≥ B1 on task metrics with token costs reported (target ≥5× fewer context tokens). B2 > B0 alone proves nothing and is never reported as synergy. If B2 ≤ B1 at B06, the finding is "parameter memory not yet beating text memory" and the next session is diagnosis, not features — the exact mirror of the rule-flip discipline.

## Metrics (pre-registered at B02, deterministic scoring only)
stage_accuracy (exact/structured match on current-milestone questions), stale_claim_rate (planted outdated facts repeated vs flagged), contradiction_handling_rate (open contradiction in AM ⇒ answer must qualify/ask, scored by structured answer fields), test_grounded_rate (recommendations citing current verified state), token_cost per task, answer_drift across sessions (same question, same state ⇒ same structured answer). Task corpus: generated from this repo's own BUILD_REPORTs and audit history — answer keys are deterministic by construction. Traps included (e.g., the dead "completion only passes at t=2" claim planted in B1's log window; correct behavior is flagging it stale via S05/A16 state).

## Sessions
**B01 — Event substrate.** `am apply` batch path, EG-1 grammar, provenance envelope, staging sidecar + lifecycle, rejection reports, determinism + fuzz tests. No LLM, no network. (Prompt written: AM_B01_PROMPT.md.)
**B02 — Eval harness first.** Task corpus + answer keys from repo history; B0/B1 lanes runnable; B2 lane stubbed pending B03; scoring engine deterministic; metrics above frozen in `eval/METRICS.md`. Pinned local model via the dormant Ollama seam, temp 0, version recorded.
**B03 — Context compiler.** Snapshot → deterministic state summary: active concepts, strong links, open contradictions (min-axis cert), low-certainty axes, goals, recent mutation causes; stable ordering; token budget ≤1,200 enforced; golden test: same snapshot ⇒ byte-identical compiled context. Contradiction-aware directive block (open contradiction ⇒ "do not present as settled; ask for a test or propose disambiguation").
**B04 — Writeback + test ingestion.** Distiller from LLM output to proposed `llm_claim` events (staged); parsers from `cargo test`/sweep/world-run artifacts to `test_verified` events (auto-commit). No raw text ever stored in AM.
**B05 — Differential mode + dashboard.** plain/AM-informed/compare lanes; shows used rows, open contradictions, proposed events with apply/reject wired to staging. Explicitly an inspection surface — never cited as proof. "Diff trivial" is a first-class reported outcome.
**B06 — Pre-registered eval run + report.** Full B0/B1/B2 on the frozen metrics. Gate applies (see Baselines).

## Deferred / labeled
Planning handoff (doc item 6) is scaffolding-until-P09 and every plan produced under it is provenance-tagged `llm_claim`; goal_push from the LLM stages like any claim. AM-backed retrieval (item 8) rides on the B03 compiler (activation-selected files/summaries, bounded, deterministic given snapshot); no vector DB, ever.

## Standing audit posture for B-track
Every session ships as a zip; independent audit reproduces tests, greps for drift (network deps outside the one designated Ollama client module, raw-text storage, staging bypasses, LLM imports anywhere in core), and verifies determinism of every AM-side artifact. Weakened metrics or post-hoc metric edits are the B-track equivalent of test weakening: forbidden (L15 spirit).
