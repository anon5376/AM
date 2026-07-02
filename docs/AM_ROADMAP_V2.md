# AM Roadmap v2 — from parameter memory to a grounded cognitive loop

Status baseline: AM 0.0.1 exists, 9/9 kill tests independently reproduced, memory is `M,W,a,b,V,u`, LLM outside the core. This roadmap turns that substrate into an agent that learns transferable world structure from interaction, with no LLM in the runtime loop.

One-line thesis (kept): a deterministic, inspectable, self-updating cognitive loop where memory, prediction, planning, and learning are native algorithms; LLMs may assist at the edge; core intelligence comes from grounded interaction and prediction-error-driven model growth.

## What this version fixes in the previous draft

1. **Numbering collision.** The draft used AM002–AM015 both as module names and (differently) in its build order. Fixed: releases keep `AM 0.x` version strings; capability milestones are `M0–M13`; world difficulty tiers are `W0–W4`. Modules are Rust module names.
2. **Missing lineage.** The draft cited MuZero/Dreamer/ReAct/Voyager (all real, correctly cited) but missed the actual ancestors of this exact design: Drescher's schema mechanism, object-oriented MDPs, Dyna-style replay, R-max exploration. Without that lineage you also miss their documented failure modes — which are precisely this project's biggest risks. Imported below.
3. **Hard problems listed as "risks" with no mechanism.** Object identity, concept splitting, credit assignment, and information gain were named but not solved. Each now has a concrete, computable v1 mechanism (D1–D8 below).
4. **Proof structure.** One transfer demo doesn't prove the module stack; a trivially memorizing agent can pass it. Fixed with a ladder: each world tier is co-designed so a specific module is *necessary*, verified by ablation, plus a negative control (rule-flip worlds) that memorization fails.
5. **Redundant module.** "Working Memory" as a separate subsystem is cut — it already exists as the activation field `a` + goal rows + a thin executive struct.

## Honest lineage and imported failure modes

- **Drescher, *Made-Up Minds* (MIT Press, 1991)** — schema mechanism: context→action→result schemas learned by marginal attribution, synthetic items for hidden state. This roadmap is closest to a modern, parameter-memory rebuild of it. **Imported failure mode:** schema/context combinatorial explosion and spurious attribution. Mitigation: hard schema budgets, minimum-evidence gates, statistical specialization tests (M6).
- **Diuk, Cohen & Littman, "An Object-Oriented Representation for Efficient RL" (ICML 2008)** — OO-MDPs learn object-class transition rules in grid worlds; direct precedent for typed relations + effect schemas. **Imported lesson:** object-class abstraction is what buys transfer; instance-level learning doesn't transfer.
- **Sutton, Dyna (ICML 1990 / SIGART 1991); Moore & Atkeson, prioritized sweeping (MLJ 1993)** — idle-tick replay/consolidation is Dyna with prioritized sweeping, not a novel "sleep" mechanism. Named honestly, implemented as such (M9).
- **Brafman & Tennenholtz, R-MAX (JMLR 2002)** — optimism under uncertainty gives a *computable* information-gain bonus: low-evidence schemas get tried. Replaces the draft's undefined "information_gain."
- **MuZero (arXiv:1911.08265), Dreamer (arXiv:1912.01603)** — kept as the scaled learned-model references; AM's differentiator is not performance but full inspectability of concepts, relations, schemas, uncertainty, and every mutation.
- **Options framework (Sutton, Precup & Singh, AIJ 1999)** — skill chunking (M11) is options discovery over learned schemas; use their termination/initiation formalism instead of inventing one.
- ReAct (2210.03629), Voyager (2305.16291), RAG (2005.11401), Transformer (1706.03762) — kept for the comparison section; all verified real.

Citation audit of the draft: all five arXiv references check out; no fabricated sources. One sloppy attribution (Dreamer's "latent imagination" cited to the MuZero paper) fixed above.

## Binding architecture decisions (the ones the draft dodged)

**D1 — Settle stays W-only.** Typed relations do NOT feed the settle dynamics in v1. The calibrated AM001 attractor loop remains untouched; typed relations are consumed by prediction and planning. Revisit only after M10 ships.
**D2 — Typed relations are a separate parameter store,** not an extension of W: sparse typed edge list `R`, deterministic BTreeMap ordering, same mutation-logging / eps_log / snapshot / determinism laws as M and W. No SQL, no strings as predicates — a closed enum.
**D3 — Percept prototypes get their own block `P` (n × F, F=12 appearance features),** under the same laws. AM001's 48 axes stay cognitive; appearance lives in P. Position/motion are track state, never prototype state (else identity collapses into appearance).
**D4 — Credit assignment is architectural, not learned:** learn only one-step effect schemas (context, action → immediate typed effects); long-horizon credit is produced by the planner chaining one-step predictions. No value functions, no eligibility traces in v1. This converts the hardest learning problem into a search problem.
**D5 — Information gain is computable:** `info_bonus(schema) = k_info / sqrt(1 + evidence_count)`. R-max-flavored optimism, printable, tunable.
**D6 — Exploration is deterministic by default:** uncertainty-greedy with fixed action-order tie-breaks (N,S,E,W,pickup,drop,open,wait). Optional ε-exploration uses a seeded PRNG owned by the experiment harness, seed recorded in the trace — never inside the core.
**D7 — Semantics are quarantined.** The world engine knows "key/door/food"; the observation serializer must not emit them. Observations carry only neutral features (shape, color, size, position, energy, reward, blocked). A lint test enforces the vocabulary whitelist, and core labels are opaque (`trk_7`, `cpt_104`). Any semantic constant in learner code fails CI.
**D8 — Every new store (P, R, schemas, tracks, episodic buffer) inherits AM001 law:** logged mutations with causes, eps_log filtering, deterministic iteration, snapshot versioning, byte-identical replay.

## World tier ladder (worlds co-designed with the modules they prove)

- **W0 — static, unique-appearance objects, full observability, fixed hidden rules.** Proves: perception→events, basic schema learning, planning, transfer. Identity is trivial here by design.
- **W1 — identical twins + object motion.** Proves object identity: appearance alone is insufficient; continuity tracking is necessary. Ablation `--no-identity` must degrade W1, not W0.
- **W2 — feature-confounded outcomes** (yellow object that opens doors AND a yellow object that drains energy). Proves outcome-driven concept splitting. Ablation `--no-split` must degrade W2 only.
- **W3 — partial observability (vision radius).** Proves memory persistence of off-screen state and epistemic actions (go look). 
- **W4 — rule variants per episode family** (which object class opens which barrier class is resampled). Proves hypothesis testing: agent must run disambiguating experiments, not recall fixed rules.

Negative control at every tier: a **rule-flip** variant where hidden mechanics are resampled; an experienced agent's advantage must shrink toward (or below) zero, proving that transfer came from learned rules, not generic exploration efficiency.

## Milestones → Codex prompts

- **M0 = Prompt 01** — AM001.1 hardening: one default theta passing all kill tests at t=20 (sweep harness), per-axis certainty display, snapshot format versioning.
- **M1 = Prompt 02** — W0 world harness: deterministic seeded grid engine, hidden-rule table, neutral observation serializer, episode runner, golden-trace tests, semantic-leak lint.
- **M2 = Prompt 03** — Perception bridge: observation → percepts (F=12) → validated AM001 Events; opaque labels; novelty from prototype mismatch.
- **M3 = Prompt 04** — Object identity: track store, pinned identity score, greedy deterministic matching, lifecycle; W1 twin tests.
- **M4 = Prompt 05** — Concept formation: leader clustering over P; outcome-driven splitting with statistical gates and budgets; W2 tests.
- **M5 = Prompt 06** — Typed relational memory `R`: enum relation types; spatial/perceptual relations (Near, Blocks) learned from statistics; causal types reserved for schemas.
- **M6 = Prompt 07** — Action schemas: Drescher-style context→action→effects with marginal-attribution-lite specialization, explosion budgets, no hardcoded semantics.
- **M7 = Prompt 08** — Prediction engine: best-schema effect prediction with confidence; surprise computation; prediction-accuracy learning curves; contradiction wiring for schemas.
- **M8 = Prompt 09** — Planner + goals: beam search over imagined effect chains; scoring with D5 info bonus; explore/exploit mode switch; anti-dither replanning rule.
- **M9–M10 = Prompt 10** — Full loop integration + Dyna-style prioritized replay + the decisive transfer benchmark with negative controls and pre-registered ablations.
- **M11+ (deferred, prompts 11+):** hypothesis engine as explicit disambiguation planning (W4), skill chunking via options, richer episodic memory, language binding (words ↔ existing concepts/schemas; language is I/O, never the brain).

## Mechanics (corrected, concrete)

**Identity (M3).** `id_score = 0.55·cos(P_track, feat) + 0.30·exp(−d²/2) + 0.15·history`; match threshold 0.6; greedy assignment by descending score, ties → lower track id; tracks: tentative(1 obs) → confirmed(3) → lost(3 missed). All constants in an `IdTheta` struct, overridable, hashed into reports.

**Concepts (M4).** Leader clustering: percept joins concept if cos ≥ 0.90 else allocates; prototype drag η_p = 0.2. **Split rule:** for concept c and action a with evidence ≥ 12: for each binary feature, if success-rate difference between feature-present/absent subsets ≥ 0.5 (each subset n ≥ 5), split on the max-gap feature. Budgets: ≤ 2 splits per concept per 100 episodes; global concept cap. Merges reuse AM001 GC.

**Relations (M5).** `Relation{subject, rtype, object, strength, confidence, variance, evidence, last_touched}` with `rtype ∈ {Near, Blocks, Opens, Causes, Enables, Requires, Contains, SameAs}`. Near/Blocks learned from spatial and movement-failure statistics; Opens/Causes/Enables written only by the schema learner (M6+) — perception never asserts causality.

**Schemas (M6).** `ActionSchema{action, context: Vec<Condition>, effects: Vec<Effect>, succ, fail, evidence, variance, expected_reward_delta, cost}`. Effects are a closed typed vocabulary: PosDelta, Blocked, HoldingSet, HoldingCleared, EntityRemoved, TilePassableSet, EnergyDelta(sign-bucket), RewardDelta(sign-bucket), NoChange. Learning: start over-general per action; specialize by adding the condition with the largest statistically-gated success-rate gap (same test as concept splitting); S_max = 40 schemas per action; prune by `evidence · exp(−age/T_prune)`.

**Prediction (M7).** Best matching schema = highest context-match score, deterministic tie-break; output effect set + confidence = succ/(succ+fail) shrunk by low evidence. After acting: per-effect error → schema stats update, variance spike, contradiction open on alternating outcomes (same semantics as AM001 P5). Surprise = Σ|error|, logged, drives replay priority and replanning.

**Planner (M8).** Beam width 8, depth 5 (PlanTheta). Imagined state: (pos, holding, energy bucket, believed passability map, visited hash). `score = 0.5·goal_progress + 0.3·expected_reward − 0.4·risk − 0.05·cost − 0.1·uncertainty + 0.15·info_bonus`, info_bonus per D5 and only weighted in explore mode; mode = explore iff no candidate plan reaches goal with confidence > 0.5. Replan only when surprise > θ_replan or plan exhausted (anti-dither). Loop safety via visited-state hashing in the beam.

**Learning loop + consolidation (M9–M10).** Per tick: observe → percepts → identity → events → settle (working memory = active set + goal rows) → predict → plan/act → outcome diff → schema/relation/concept updates. Idle ticks: replay top-16 transitions by priority |error| (prioritized sweeping), decay priority ×0.9 after replay, merge/decay via existing AM001 P6. Learning ≠ storing: every update changes future predictions or action selection, and each has a falsifier (the prediction it implies).

**Transfer benchmark (M10).** 20 seeded map pairs per tier. Success criteria (targets; tune via theta files, never weaken the test): experienced median steps-to-exit ≤ 0.7 × fresh median on W0; rule-flip control: experienced ≥ 0.9 × fresh (advantage vanishes); pre-registered ablations each degrade exactly their designed tier: no-identity→W1, no-split→W2, no-relations/schemas→all, no-planner→all, no-replay→slower convergence. Every run emits trace hashes; whole experiments are replayable.

## Comparison section (kept, trimmed of rhetoric)

Versus LLMs: AM has persistent inspectable state, online updates, auditable belief changes, consequence-grounded learning, no cloud dependency; LLMs keep vastly broader priors and instant open-domain competence — AM starts dumb and narrow. Versus RAG: AM memory is operative state, not retrieved text, with first-class forgetting/uncertainty/contradiction; RAG remains better for document QA at scale. Versus ReAct/Voyager agents: AM's planning and skills are testable structures with numeric confidence, not prompted traces; LLM agents are immediately competent on borrowed knowledge. Versus MuZero/Dreamer: AM trades raw statistical power and perception for full inspectability and auditability. All four inferiority lists from the draft stand — they were correct.

## Risks (kept, plus imported ones) and mitigations

Schema/context explosion (Drescher) → budgets + evidence gates + pruning, tested. Spurious attribution → statistical gates with minimum subset sizes; contradiction machinery on schemas. Object identity ambiguity → W1 tier isolates it; identity confidence exposed. Hardcoded semantics leaking in → D7 lint as CI. Toy-test theater → tier-ablation design + rule-flip negative control; a passing script that doesn't move these metrics is defined as fake progress. Memory corruption / silent mutation → diff-integrity fuzz extended to every new store. Stale links / reused-row corruption → snapshot versioning + row-generation counters (Prompt 01). Transfer illusion from generic exploration → the rule-flip control exists precisely for this.

## Never build / defer

Never in this program: LLM in the runtime loop; semantic constants in learners; SQL/vector-DB memory; reward hand-crafted per map (only the fixed env signals). Defer past M10: hypothesis engine, options/skills, partial observability (W3), counterfactuals, language binding, any neural perception. Language remains I/O bound to existing structures — it must never be required for the transfer proof.
