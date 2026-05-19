# PLAN

## Goal

Find or disprove a storage primitive that lowers Gear/Sails VFT operation gas beyond the current optimized static-table candidate.

## Current Strategy

Use the current `SailsStatic` VFT fast path as the baseline, then score the already implemented million-entry layout candidates before adding new variants. If those candidates fail the replacement threshold, write the no-winner decision instead of widening the experiment set.

## Phases

- [x] Inspect current VFT/storage benchmark state.
- [x] Establish current `SailsStatic` baseline.
- [x] Score existing million-entry candidates.
- [x] Run the fast feedback check and record results.
- [x] Record at least three credible candidates or rejections.
- [x] Run large-capacity and gas-profile checks for survivors.
- [x] Write final `RESULTS.md`.
- [x] Run final verification.

## Open Decisions

- 1 million entries is sufficient for this pass because `CONTROL.md` sets `large_capacity_target: 1_000_000_entries`.
- No winning variant cleared the replacement threshold; recommendation is experimental, not immediate public default.
