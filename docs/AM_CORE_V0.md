AM-Core v0 — Parameter-Memory Attractor Engine

Deliverable for the three demands: (1) the algorithm itself, (2) memory that IS the parameters, (3) total inspectability as signed scalars on named axes.

Tag legend: [KM] known-math · [NT] new-but-testable · [SP] speculative · [IMP] impossible-as-stated.

1. Complete state definition

Everything the mind is lives in six arrays plus scalar constants. Nothing else exists. Persistence = serializing these arrays to disk (mmap or flat file). There is no database of memories. [KM: it's arrays]

D     = 48        # named axes (hand-defined in v0)
N     = 4096      # concept slots (grows by allocation)
K     = 32        # max Hebbian links per concept (sparsity)
A_max = 64        # max simultaneously active concepts (k-WTA cap)

AXES : list[str], len D
  # starter set (extend to 48):
  # urgency, valence, arousal, agency, self_relevance, user_relevance,
  # goal_relevance, temporal_near, temporal_past, concreteness, social,
  # risk, effort, value, novelty, stability, truth_assert, desire,
  # obligation, completion, familiarity, emotional_charge, scope, priority
  # NOTE: "certainty" is NOT a stored axis — it is derived from V (see §3.P3)

M : float32[N, D]                 # concept coordinates. THE memory.
W : sparse float32[N, N]          # Hebbian links, ≤K nonzero per row, W_ij ∈ [0, w_max]
a : float32[N] ∈ [0, 1]           # activation field = the current thought
b : float32[N] ∈ [0, b_max]       # consolidation baseline (importance), slow variable
V : float32[N, D]                 # exponentially-weighted per-cell variance (uncertainty)
u : int64[N]                      # last-touched tick

L : dict[int → str]               # labels. Inspection only — zero role in dynamics.
G : list[goal_row_id]             # rows currently registered as goal attractors
open_contradictions : list[(i, d, evidence_ids)]

THETA (all logged, all in one struct):
  lam_a=0.70   fast activation decay        lam_b=0.999  slow baseline decay
  lam_settle=0.55  settle leak              lam_V=0.95   variance EW decay
  eta_M=0.15   coordinate write rate        eta_W=0.05   hebb rate
  del_W=0.002  link decay                   beta=1.4     settle gain
  gamma=0.9    global inhibition            T=20         max settle iters
  eps=1e-3     fixpoint tolerance           th_act=0.25  active threshold
  th_write=0.40 write gate                  th0=0.5      sigmoid offset
  tau_g=0.5    goal kernel width            a_init=0.6   new-concept activation
  rho_b=0.01   consolidation gain           rho_c=0.15   contradiction pressure
  th_V=0.35    contradiction variance gate  m=5          sign-alternation window
  A_old=5000   staleness age                th_gc=0.05   garbage-collect baseline
  th_merge=0.92 merge similarity            w_max=1.0    link cap
  sigma=0      noise (OFF ⇒ deterministic)

Footprint: M+V ≈ 1.6 MB, W ≈ 0.5–1.5 MB. The whole mind is < 5 MB and fits in CPU cache. Determinism guarantee: fixed iteration order, float32, single-threaded core path, σ=0 ⇒ same snapshot + same event stream = byte-identical trace.

Goals are ordinary rows (fully visible in dumps) whose ids are registered in G; their M row is the attractor point. No hidden goal representation.

2. Input interface

The core never sees text. A parser (regex in week 1; local LLM later — irrelevant to the core) emits Events:

Event = {
  id:       int,
  cues:     [(concept_ref, strength ∈ (0,1])],           # light these up
  asserts:  [(concept_ref, {axis_name → value}, weight)], # evidence: drag coordinates
  links:    [(concept_ref, concept_ref, hint ∈ [-1,1])],  # optional explicit association
  goal_ops: [push(goal_ref) | pop(goal_ref)]
}
concept_ref = existing id | new label → allocate row

Allocation: M[new] filled from asserted axes (others 0), V[new]=v0, b[new]=b0, L[new]=label. Contradictory input like "rust chosen" then "rust rejected" arrives as opposite-sign asserts on truth_assert for the same row — the contradiction mechanism (§3.P5) runs on that.

3. The update rule: state(t) + event → state(t+1)

Seven phases, all deterministic, every mutation logged with its cause at the mutation site.

P1 — Cue injection. For each cue (i, s): a_i ← max(a_i, s), u_i ← tick. Asserted concepts also get a_i ← max(a_i, a_init) so they pass the write gate. New labels allocate rows.

P2 — Settle. Recall is dynamics, not lookup. [KM: sparse attractor settling + k-WTA + spreading activation — Hopfield 1982; Collins & Loftus 1975; k-winners classic]

repeat ≤ T times, stop when max|Δa| < eps:
    goal_i = Σ_{g∈G} b_g · exp( −‖M_i − M_g‖² / tau_g )     # goals pull nearby concepts
    net_i  = Σ_j W_ij·a_j + b_i + goal_i − gamma·mean(a)     # links + baseline − inhibition
    a_i    ← clamp( lam_settle·a_i + beta·sigmoid(net_i − th0), 0, 1 )
then k-WTA: keep top A_max activations ≥ th_act, zero the rest.
A = active set = the assembled thought.

A partial cue falls into a basin and completes the full assembly. No query executed; nothing was "retrieved from" anywhere. This is pattern completion.

P3 — Write. Memory = moving coordinates. [KM: delta rule / online centroid update]

for (i, targets, w) in event.asserts, if a_i ≥ th_write:
    for (d, v) in targets:
        Δ = eta_M · w · a_i · (v − M[i,d])
        M[i,d] += Δ                                   # log: ("M", i, d, Δ, cause)
        V[i,d] ← lam_V·V[i,d] + (1−lam_V)·(v − M[i,d])²   # EW variance

Consequences that fall out of the math, for free:

Repetition = repeated small drags toward the same target ⇒ reinforcement without duplication. There is no second row to create.

Opposing evidence = drags in opposite directions ⇒ coordinate settles near the midpoint while V[i,d] spikes ⇒ uncertainty is a printable number. Derived certainty: cert_i = 1/(1+mean_d V[i,d]).

P4 — Hebbian link update. [KM: Hebb + decay (Oja-flavored), prune to top-K]

for i, j in A×A, i≠j:
    W_ij ← clip( W_ij + eta_W·a_i·a_j − del_W·W_ij , 0, w_max )
explicit event.links add eta_W·hint on top
prune entries < eps_W; keep top-K per row

Concepts thought together wire together. Links are the associative structure that P2 settles over.

P5 — Contradiction pressure. [NT — the invented part; calibration of th_V, rho_c is an open experiment]

flag (i,d) if V[i,d] > th_V AND last m writes to (i,d) alternate sign
on flag:  b_i += rho_c            # pressure: stays hot in future settles
          open_contradictions.append((i, d, evidence_ids))
resolve:  a run of same-sign writes shrinks EW-variance below th_V_close
          ⇒ close record, relax b_i

Contradiction is not a symbolic rule — it is a variance spike with sign alternation, detected numerically, and it manifests as sustained activation pressure until evidence resolves it.

P6 — Decay and true forgetting. [KM decay; merge policy NT]

a ← lam_a · a
b_i ← lam_b · b_i + rho_b · a_i          # often-thought concepts consolidate
for rows with (tick − u_i > A_old) and (b_i < th_gc):
    j = nearest row by axis-cosine, if sim > th_merge:
        M_j ← (b_j·M_j + b_i·M_i)/(b_j+b_i)      # weighted merge
        W row/col OR-merged;  L alias old→j;  free row i
    else: free row i                              # coordinates cease to exist

Forgetting is actual information loss in the parameters, not a deleted-flag on a database row.

P7 — Snapshot. Append (tick, A, all deltas with causes) to an append-only trace log. The dynamics never read this log — it exists for audit, diff, and the later neural-training dataset only.

Daemon tick = the same step with an empty event (P2, P5, P6, P7 only). Cost is identical math, zero LLM.

4. Runnable pseudocode — one complete cognitive step (~70 lines)

# state S: M[N,D], W sparse, a[N], b[N], V[N,D], u[N], L, G, open_contradictions
# THETA: constants from §1.  sigma=0 ⇒ deterministic.

def step(S, event, tick, log):
    # ---- P1: cue --------------------------------------------------
    for ref, s in event.cues:
        i = resolve(S, ref, log)                 # id lookup or row allocation
        S.a[i] = max(S.a[i], s); S.u[i] = tick
    for ref, targets, w in event.asserts:
        i = resolve(S, ref, log)
        S.a[i] = max(S.a[i], THETA.a_init); S.u[i] = tick

    # ---- P2: settle (recall = attractor dynamics) -------------------
    for t in range(THETA.T):
        goal = goal_pull(S)                      # Σ_g b_g·exp(−‖M_i−M_g‖²/tau_g)
        net  = S.W @ S.a + S.b + goal - THETA.gamma * S.a.mean()
        new_a = clamp(THETA.lam_settle*S.a + THETA.beta*sigmoid(net-THETA.th0), 0, 1)
        done = (abs(new_a - S.a).max() < THETA.eps)
        S.a = new_a
        if done: break
    kwta(S.a, THETA.A_max, THETA.th_act)         # top-A_max survive, rest → 0
    A = active_set(S.a, THETA.th_act)            # the assembled thought

    # ---- P3: write (memory = moving coordinates) --------------------
    for ref, targets, w in event.asserts:
        i = resolve(S, ref, log)
        if S.a[i] < THETA.th_write: continue
        for d, v in targets.items():
            delta = THETA.eta_M * w * S.a[i] * (v - S.M[i, d])
            S.M[i, d] += delta
            S.V[i, d] = THETA.lam_V*S.V[i, d] + (1-THETA.lam_V)*(v - S.M[i, d])**2
            log.mut("M", i, d, delta, cause=("write", event.id, w, S.a[i]))

    # ---- P4: hebb ----------------------------------------------------
    for i in A:
        for j in A:
            if j == i: continue
            dw = THETA.eta_W*S.a[i]*S.a[j] - THETA.del_W*S.W[i, j]
            S.W[i, j] = clip(S.W[i, j] + dw, 0, THETA.w_max)
            log.mut("W", i, j, dw, cause=("hebb", S.a[i], S.a[j]))
        for (p, q, hint) in event.links:
            if p == i: S.W[i, resolve(S, q, log)] += THETA.eta_W * hint
        prune_topk(S.W, i, THETA.K)

    # ---- P5: contradiction pressure ----------------------------------
    for (i, d) in recent_write_cells(log, window=THETA.m):
        if S.V[i, d] > THETA.th_V and signs_alternate(log, i, d, THETA.m):
            if not is_open(S, i, d):
                S.open_contradictions.append((i, d, log.evidence(i, d)))
                S.b[i] += THETA.rho_c
                log.mut("b", i, None, THETA.rho_c, cause=("contradiction_open", d))
    for c in list(S.open_contradictions):
        if S.V[c.i, c.d] < THETA.th_V_close:
            S.open_contradictions.remove(c)
            S.b[c.i] -= THETA.rho_c
            log.mut("b", c.i, None, -THETA.rho_c, cause=("contradiction_close", c.d))

    # ---- P6: decay / forget -------------------------------------------
    S.a *= THETA.lam_a
    S.b  = THETA.lam_b*S.b + THETA.rho_b*S.a
    for i in stale_rows(S, tick, THETA.A_old, THETA.th_gc):
        j = nearest_axis_neighbor(S, i, THETA.th_merge)
        if j is not None: merge_rows(S, i, j, log)   # weighted-avg M, OR links, alias L
        else:             free_row(S, i, log)        # true forgetting

    # ---- P7: snapshot ---------------------------------------------------
    log.snapshot(tick, A, S)      # append-only; dynamics never read it
    return S

Helper contracts: resolve = label→id map with allocation; kwta = zero all but top-k; merge_rows/free_row log every cell they touch. Full reference implementation ≈ 300 lines NumPy including CLI.

5. Complexity (per step)

Phase

Time

With defaults

P2 settle

O(T·(nnz(W) + |G|·N))

20·(4096·32) ≈ 2.6M mul-add ⇒ < 1 ms CPU

P3 write

O(|asserts|·D)

negligible

P4 hebb

O(|A|² + |A|·K)

64² ≈ 4k updates

P5–P6

O(|A| + stale rows)

negligible

Space

O(N·D + N·K)

< 5 MB

Daemon tick: same bound, no P3/P4 event loops. LLM cost inside the core: zero.

6. Memory substrate characterization (the demanded four)

Write op: coordinate drag (P3) + link change (P4). No insert operation exists.Read op: cue → settle → active assembly + its coordinates. No query operation exists.

Capacity [KM for the theory frame, empirical for this hybrid]: Coordinates are localist — N·D exact scalars, no row-to-row superposition loss. The binding constraint is assembly capacity in W: dense Hopfield stores ≈ 0.138·N patterns (Amit–Gutfreund–Sompolinsky 1985); sparse-activity associative nets scale far better, ≈ N/(p·|ln p|) with activity fraction p (Tsodyks–Feigel'man 1988). This design is a localist/sparse hybrid, so the honest statement is: exact capacity unknown, basin overlap breaks first, measure it (kill test #2 scales N and counts wrong-basin completions).

Interference / forgetting behavior: (a) basin overlap → wrong assembly completes = false recall; (b) merge policy folds two concepts that were actually distinct = aliasing; (c) parser maps two referents to one row = coordinate thrash. All three are visible in the diff log — that is the payoff of inspectability. Forgetting = λ-decay + GC merge = designed, tunable, observable information loss.

Deliberately excluded: modern Hopfield networks (Ramsauer et al. 2020). Their retrieval is softmax attention over stored patterns — i.e. a similarity query over a pattern store, which quietly re-smuggles the forbidden vector-DB shape under prettier math. Classic sparse attractor + localist rows keeps "memory in parameters" honest. [KM, decision NT]

Known gap — episodic memory: M/W store semantic gist. "What did I say on Tuesday" lives only in the audit log, which is outside the parameters. v0 accepts gist-only; a fast-weight episodic matrix is the later fix. Stated so nobody discovers it in month three.

7. Relations — the expressiveness gap, stated plainly

Axis coordinates express properties. Propositions ("user WANTS rust" as structure, not vibes) need typed relations. v0 ships with only W links + relation-tagged event records — associative, not propositional. v0.2 adds linear relational operators: learn R_type ∈ R^{D×D} minimizing ‖R_type·M_subj − M_obj‖² [KM: Linear Relational Embedding, Paccanaro & Hinton 2001]. Relations become inspectable D×D matrices in the same axis space; relational recall = apply R, settle near the image. Without this the system is an associative memory and must not be sold as a reasoner.

Why not VSA/hypervector binding for relations: 10k-dim bound vectors are unreadable — superposition destroys per-dimension nameability, which kills demand #3. [IMP: HV state + [+1.3] readability simultaneously] HV is permitted later only as a fast candidate-index, never as state.

8. Inspectability contract

The state is nothing but M, W, a, b, V, u — printable scalars. Any state that can't be printed doesn't exist here by construction.

am dump [--sort axis|act|b] — every concept:

AM001        [goal_rel +1.9  agency +1.2  truth_assert +0.8  valence +0.6]  a=0.71 b=0.42 cert=0.83 age=12
rust         [goal_rel +1.4  effort  -0.7  truth_assert +0.3]               a=0.55 b=0.31 cert=0.41 age=12
   ⚠ contradiction open: truth_assert (ev #382 vs #391)

am step --diff — every changed cell with its cause, emitted at the mutation site:

M[rust][truth_assert]  -0.11   ← write ev#391 (w=0.9, a=0.77)
V[rust][truth_assert]  +0.09   ← variance update
W[rust↔am001]          +0.031  ← hebb (0.77·0.81)
a[deadline]         0.44→0.31  ← decay λ_a
b[rust]                +0.15   ← contradiction_open(truth_assert)

am watch — live TUI (rich/ratatui): rows = concepts, columns = axes as signed bars, activation heat column, cells flash on delta. Strictly read-only: renders from snapshots, cannot mutate.

Axis admission rule for any learned axis later: it enters M only if the top/bottom 20 concepts along it are human-nameable; otherwise it stays a hidden feature and is refused. [NT]

9. Honesty ledger

Demands satisfied: algorithm = §3–4 (one update rule, ~70 lines, complexity in §5); memory-as-parameters = §1+§6 (no store/retrieve API exists in the codebase — only arrays and the step function); inspectability = §8 (state is exclusively printable signed scalars on named axes).

Lineage, admitted: Hopfield 1982 attractor settling · Hebb + decay · delta rule · k-WTA · spreading activation (Collins & Loftus 1975) · Linear Relational Embedding (Paccanaro & Hinton 2001) · sparse-coding capacity theory (Tsodyks–Feigel'man 1988). The novelty is the combination — named-axis localism + variance-as-contradiction + cause-tagged parameter diffs + goal-rows-as-attractors — not the math. Zero-lineage novelty does not exist; anyone offering it is selling. [IMP as originally phrased; this is the honest maximum]

What v0 will actually do: persistent drift-consolidated memory across restarts · cue-completion recall · reinforcement without duplication · numeric uncertainty · contradiction flags with pressure · true forgetting · a live-visible thought field.What it will NOT do [SP if claimed]: multi-step reasoning, planning beyond goal-pull, language. The LLM stays outside as parser/renderer under your seam rules and cannot touch M, W, a, b, V directly — it can only emit Events, which pass through the same validated step as everything else.

10. Kill tests — alive or fake

Determinism: same snapshot + same event stream ⇒ byte-identical trace. Fail = the core is not an algorithm.

Completion: teach 30 assemblies; cue with 30% of one ⇒ correct assembly ≥ 90%, wrong-basin < 5%. Scale N to find the real capacity curve.

Reinforcement: identical fact ×10 ⇒ one row, monotone b, zero duplicates.

Contradiction: opposing asserts ⇒ V spike + open flag + midpoint coordinate; a same-sign evidence run closes it.

Forgetting curve: untouched concept's recall probability decays per design, measurably.

Diff integrity (fuzz): every changed cell has exactly one cause entry; no silent mutation path exists.

Any test unfixably failing at this scale ⇒ the idea is fake; kill it before building scaffolding on top.

11. Build order

Reference implementation in NumPy first (~300 lines, days not weeks — the math must be provable before it's portable); freeze THETA and the step contract once kill tests 1–4 pass; port runtime + daemon to Rust at 0.0.2 unchanged. Rust-before-math is scaffolding-first, which is the documented failure mode.