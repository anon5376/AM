Mission

Build the falsification machinery BEFORE any synergy feature exists: task corpus with deterministic answer keys, lanes B0/B1 (B2 stubbed), deterministic scoring, and an LLM transport boundary that makes every test net-free and byte-reproducible. If this session is sloppy, every later Track B claim is unfalsifiable.

Module layout

src/beval/ (name avoids clashing with existing src/eval/ sweep machinery; do not touch that module). CLI: am beval --corpus <dir> --lane b0|b1|b2 --transport replay|live --fixtures <dir> [--record] --out <results.json>.

LLM transport boundary (the load-bearing design)

Trait LlmTransport { fn complete(&mut self, prompt: &str) -> Result<String> } with exactly two impls:

- ReplayTransport: reads fixture files keyed by sha256(prompt) from --fixtures. Missing fixture = hard error naming the hash and the first 120 chars of the prompt. ALL tests use replay. Committed fixtures for tests are SYNTHETIC (hand-written plausible responses, clearly labeled "synthetic": true in a sidecar manifest) -- they prove plumbing and scoring, never model quality.
- OllamaTransport: lives in src/llm/ (the existing dormant seam) only. Config pins model name AND version/digest, temperature 0, seed if the API supports it; all three recorded into results JSON. --record mode runs live and writes fixtures (manifest marks "synthetic": false). Live mode is NEVER exercised by cargo test.
- Drift-scan amendment: the B01 network-crate blacklist over Cargo.toml is replaced by a scoped rule -- exactly one minimal HTTP client dependency is permitted, and a source scan asserts it is imported ONLY within src/llm/. Any other module importing it fails the drift test. Everything else in the blacklist stays forbidden.

Corpus (v1, committed under beval/corpus/)

25-40 tasks as JSON: {id, category, question, answer_key, source_ref}. Categories and minimum counts: stage_accuracy >=8 (current default theta hash, eta_w value, test counts, session states -- answers derived from committed docs), stale_claim >=5 (traps: outdated facts like "completion passes only at t=2" or "del_w default is 0.002" planted verbatim into the B1 log fixture; key = response must NOT repeat the stale fact and must reflect current state), contradiction_handling >=5 (tasks where the AM snapshot given to B2 has an open contradiction; key = structured answer must qualify/flag, scored by required marker fields -- B2-only tasks are marked requires_lane: b2 and skipped elsewhere without penalty), test_grounded >=5 (recommendation questions whose keys demand citing a current verified artifact name/value), drift >=3 (same question asked twice in one run; keys require identical structured answers).

Answer keys are deterministic matchers only: exact, regex, must_contain_all, must_contain_none. No LLM-as-judge, no embedding similarity, no human scoring. Every task carries source_ref to the committed doc line(s) justifying the key.

Responses are elicited in a structured envelope: the lane prompt instructs the model to answer inside ANSWER: <one line> and matchers run against that line only; a missing envelope scores as failure with reason NoAnswerEnvelope.

Lanes

- B0: bare -- task question only.
- B1: question + raw-log context: beval/fixtures/rawlog_v1.md, a committed excerpt-compilation of real session history (build reports, chat-style notes) INCLUDING the planted stale facts, budget 8000 tokens. Token counting pinned: tokens = ceil(chars/4), documented; truncation is recency-window (keep tail), deterministic.
- B2: returns LaneUnavailable in B02 (context compiler is B03). Results JSON records it as such; scoring skips, never zero-fills.

METRICS.md (frozen at end of session -- post-hoc edits are L15-class violations)

Per-category accuracy = matched/total (excluding skipped-by-lane). stale_claim_rate = stale repeats/stale tasks (lower better). drift = mismatched pairs/pairs. token_cost = context tokens per task (mean, by the pinned counter). Overall table format specified. State the B06 gate verbatim from the roadmap: B2 must beat B1 with >=5x fewer context tokens or the finding is diagnosis.

Tests (all replay-transport, all deterministic)

corpus_schema_validates (every committed task parses, keys well-formed, category minimums met); scorer_matchers_deterministic (table-driven, incl. envelope-missing case); replay_double_run_byte_identical (full B0+B1 over synthetic fixtures twice -> byte-identical results JSON); token_truncation_deterministic_and_budgeted; stale_trap_scoring_correct (synthetic fixture that parrots the planted stale fact scores as repeat; one that flags it scores clean); missing_fixture_errors_cleanly; live_transport_never_in_tests (drift-style scan: no test imports OllamaTransport).

Riders

R-a: am apply rejection exit prints one clean error line, no backtrace (normal outcome, not a crash).

R-b: BUILD_REPORT_B02 includes: corpus size by category, synthetic-fixture disclosure statement, the frozen METRICS.md hash, drift-scan amendment description, and explicit statement that no live LLM ran during tests.

Forbidden

LLM-as-judge or any nondeterministic scoring; network in tests; touching src/core, src/world, src/percept, src/eval; real-model claims from synthetic fixtures; metric definitions changed after freeze; new deps beyond the single scoped HTTP client.
