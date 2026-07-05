# APPROVED SPEC AMENDMENTS v0.1

**A1 — Goal pull normalization (bug fix).** The doc's `exp(−‖M_i−M_g‖²/tau_g)` with tau_g=0.5 underflows to ~0 for any realistic 48-dim distance. Replace with mean-squared distance:
`goal_i = Σ_{g∈G} b_g · exp( −(‖M_i−M_g‖²/D) / tau_g )`, new default `tau_g = 1.0`.

**A2 — Global link decay (bug fix).** The doc decays links only inside the co-active A×A loop, so idle links never decay. Change P4 to: Hebb growth for co-active pairs `W_ij = clip(W_ij + eta_w·a_i·a_j, 0, w_max)` (no decay term in-loop), then a global pass every step: all links `w ← w·(1−del_w)`, prune `< eps_w`, keep top-K. O(rows·K), cheap.

**A3 — Settle logging granularity.** Do NOT log per-iteration activation changes (T=20 would explode the trace). Log ONE MutationRecord per row: net change from pre-P2 value to post-k-WTA value, cause=`Settle`. Iteration count goes in `StepTrace.settle_iters`.

**A4 — eps_log.** New Theta field `eps_log = 1e-6`. Any candidate mutation with `|delta| < eps_log` is neither applied nor logged (M, V, W, a, b alike). This keeps diff-integrity exact with zero near-zero noise.

**A5 — mean(a).** Inhibition term uses the mean over **allocated rows only**, not over all N slots.

**A6 — Write evidence sign.** `sign = signum(v − M[i,d]_before)`. Writes with `|v − M_before| < eps_log` are excluded from write history.

**A7 — Alternation rule.** `signs_alternate` = (history for (i,d) has ≥ 4 entries) AND (every adjacent pair among the last `min(len, m)` entries has strictly opposite sign). With the demo's write sequence +1, −1, +1, −1 this fires on the 4th write.

**A8 — lam_V default 0.8 (bug fix).** With the doc's 0.95, EW variance after 5 alternating ±1 writes reaches only ≈0.23 < th_V=0.35 — the contradiction mechanism mathematically cannot trigger inside the m=5 window. `lam_V = 0.8` gives ≈0.59 after 4 writes. Keep th_V=0.35.

**A9 — Explicit link hints are symmetric.** Apply `eta_w·hint` to both W[i][j] and W[j][i], clip to [0, w_max] (negative hint = weight reduction).

**A10 — Snapshot completeness.** Snapshot MUST include recent-write history and open contradictions; save/load/continue must be byte-identical to never-stopping. Not optional.

**A11 — u semantics.** `u` refreshes only on external touch (cue/assert). Spreading-activation recall does not refresh `u`; consolidation via `b` is what protects frequently-recalled concepts from GC. Document this.

**A12 — Determinism contract scope.** Same-machine bit-exactness (std f32, libm exp/sigmoid allowed). Cross-platform bit-exactness is out of scope for v0 — do not build fixed-point math.

**A13 — Theta override.** Every CLI command and every test accepts `--theta theta.json` / a theta file. Kill tests may ship tuned constants in `tests/theta/*.json`. Tuning constants is legitimate; changing formulas or deleting/weakening pass criteria is not. Every test and the report must record the theta (hash) actually used.

**A14 — Contradiction lifetime.** No timeout in v0: absent resolving evidence, a contradiction stays open and `b` stays boosted indefinitely. Note it in the report as designed behavior.

A15a — Sustained cue input. Each step, build an external input vector I from the event: I_i = k_i · s_i where s_i = max cue strength for cued rows, a_init for asserted rows, else 0. I is held constant across ALL settle iterations of that step and cleared afterward (never persists across ticks): a_i ← clamp(lam_settle·a_i + beta·sigmoid(net_i + I_i − th0), 0, 1). New Theta field k_i: f32 (grid below). Logging unchanged (A3 net-per-row).
A15b — Write-gate fallback (implement only if the A15a re-sweep still yields 0 all-pass): P3 gate becomes max(a_pre_settle_i, a_i) ≥ th_write, where a_pre_settle is captured after P1.
A15c — Resting-field invariant (mandatory regardless): Theta::validate() rejects any theta where beta·sigmoid(−th0)/(1 − lam_settle) ≥ th_act. New integration test resting_field.rs: allocate 40 rows with mixed b, run 30 empty ticks, assert max resting activation < th_act and zero rows in every k-WTA active set. This pins the disease so it cannot regress.

A16 — Default link decay disabled (evidence-based). Committed default del_w=0.0. Independent full-scale confirmation (n=4096, 30 assemblies, t=20) of all 14 scaled all-pass points with del_w=0.002, plus a fine sweep at del_w ∈ {1e-4, 2e-4, 3e-4, 5e-4, 1e-3}, shows completion recall collapsing 0.47 → 0.09; no nonzero del_w passes at full scale. Completion sits near an attractor-basin bifurcation: assemblies drop out whole, by training age, under any sustained W erosion. The A2 global decay pass ships alive-but-dormant; forgetting is carried by λ-decay + GC merge (verified del_w-independent across all 1296 evaluated sweep points). Scaled sweeps are filters only and provably admit del_w false positives; any default-theta change re-runs full-scale confirmation.

A17 — Default eta_w increased to 0.30 (evidence-based margin fix). S06 default eta_w=0.15 passed all criteria at t=20 but had recall_margin_095=0.133333, meaning a uniform 5% W shrink collapsed completion recall from 1.000000 to 0.133333. Independent full-scale runs (n=4096, 30 assemblies, t=20) show eta_w=0.30 with all other committed defaults unchanged is all-criteria-pass with recall_margin_095=1.000000 and contamination=0.000000. This is a single-axis move, not a re-tune: neighboring eta_w=0.30 points at other lam/beta fail other criteria. Caveat: link formation speed is doubled; the contamination metric is the spurious-link guard and stays 0.000000 at full scale.

A18 — Snapshot RECENT ring ratification. Snapshot format v5 includes `recent_mutation_causes`, a reporting-only ring used by the B03 context compiler's `RECENT` section. The ring is not cognition and is not an input to activation, recall, write strength, consolidation, contradiction pressure, certainty, decay, garbage collection, planning, or world dynamics. Capacity is fixed at 20 entries (`RECENT_MUTATION_CAUSE_CAPACITY`); overflow drops the oldest entries deterministically. The ring itself performs no deduplication: distinct mutation events remain distinct entries, including entries with the same cause but different `event_id`. The context renderer may deduplicate only for display by grouping the retained entries by `Cause` and emitting deterministic counts such as `Cue x2`; that display grouping must not mutate or truncate the ring and must preserve multiplicity by count. Every ring entry carries `tick`, `event_id`, and `cause`, so provenance/reporting code can join the entry back to the corresponding trace event id. There is no missing-id fallback in v5; older snapshots without this field are rejected by the snapshot version check rather than silently upgraded. Allowed code readers are limited to snapshot serialization/deserialization, context compilation, dashboard/provenance/reporting surfaces, and tests. Dynamics modules must not read the ring; `tests/a18_recent_ring.rs` statically guards the current dynamics paths.
