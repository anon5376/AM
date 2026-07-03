# S06 Workplan

## Mission

Finish W0 world mechanics and land the bounded S05 audit riders without adding a perception bridge, core-world runtime wiring, new required dependencies, database memory, randomness outside the existing seeded harness, or any LLM call path.

## Files And Order

1. World substrate:
   - Add `src/world/classes.rs` for the single source of opaque behavior-class constants and deterministic class lists.
   - Refactor `src/world/theta.rs` to replace reward/rule-code fields with W0 mechanics constants, entity mix, and reserved tier flags.
   - Refactor `src/world/grid.rs` for class-driven placement, rule-seed shape permutation, rule-seed portable-to-barrier matching, held state, pickup/drop/open, consumable/hazard/exit/energy/tick-cap termination, and trace fields.
   - Update `src/world/observation.rs`, `src/world/runner.rs`, `src/world/mod.rs`, and `src/cli/commands.rs` for held appearance, termination, and optional ASCII rendering.

2. Core riders:
   - Change `AmState::new` to validate theta and return `Result`.
   - Validate snapshot theta after format-version checks in `src/core/snapshot.rs`.
   - Add A16 to `docs/AM_CORE_V0_AMENDMENTS.md`.
   - Add completion `recall_margin_095` metric and sweep CSV column in `src/eval/criteria.rs` and `src/eval/sweep.rs`.
   - Add link-decay aliveness coverage.

3. Tests:
   - Update existing world golden, determinism, CLI, observation, quarantine, and core tests for the new APIs.
   - Add mechanics ground-truth tests covering blocking, matching open, failed actions, pickup/drop, energy effects, termination, tier flags, seed separation, C1 snapshot rejection, and link decay.

4. Docs and report:
   - Update `docs/W0_WORLD.md` and `docs/OBSERVATION_SCHEMA.md`.
   - Run demo commands including rendered pickup-open-exit episode and quiet core checks as needed.
   - Write `docs/BUILD_REPORT_S06.md` with commands, test results, theta hashes, regenerated golden causes, core diff scope, demo transcript, limitations, and next tasks.

## Risks

- The world mechanics use sanctioned semantic vocabulary inside `src/world`; quarantine tests must keep those names out of observations and core/learner/planner paths.
- `AmState::new` returning `Result` touches many tests and evaluation call sites; failures should be mechanical, not behavioral.
- Golden world hashes will churn because traces and observations gain termination/action-failure/held fields and mechanics replace rule-code pings.
- W0 has no solvability generator; demo and correctness tests must use pinned seeds and scripts, not a search layer.

## Deferrals

- No perception bridge.
- No schema learner.
- No planning integration.
- No solvability guarantees or map search.
- Reserved W1+ tier flags remain validation errors.
