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

