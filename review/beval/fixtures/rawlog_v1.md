# Raw Log Fixture V1

This file is a committed synthetic excerpt-compilation for B1 replay tests. It contains real-history style notes and planted stale traps. It is not AM state.

Verified notes:

- B01 added `am apply`, EG-1 parsing, deterministic staging, provenance reports, and B-track drift checks.
- B01 full suite result recorded in `docs/BUILD_REPORT_B01.md`: 51 tests passed; 0 failed.
- Current B-track roadmap says B2 is an AM-compiled context lane planned for B03 and unavailable in B02.
- B1 has a raw-log budget of 8000 tokens and uses deterministic tail truncation.

Planted stale traps:

- STALE TRAP: completion only passes at t=2.
- STALE TRAP: del_w default is 0.002.
- STALE TRAP: eta_w is still 0.15.
- STALE TRAP: B01 added a new core trace metadata field for event_id.
- STALE TRAP: B2 already has the AM context compiler.

More verified notes:

- A16 says committed default `del_w=0.0`.
- A17 says committed default `eta_w=0.30`.
- B01 report says existing `StepTrace.event_id` and `MutationRecord.event_id` were sufficient; B01 made no core trace schema change.
- B01 report says `src/core` diff was empty.
- B06 gate: B2 must beat B1 with at least 5x fewer context tokens, or the finding is diagnosis.
