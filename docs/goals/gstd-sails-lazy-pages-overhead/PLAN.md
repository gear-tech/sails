# PLAN

## Goal

Reduce or conclusively attribute fixed Gear/gstd/Sails overhead in noop and VFT hot-path execution.

## Current Strategy

Keep the stack/heap message-read change as the current winner. It does not lower the pure Sails noop floor, but it removes the allocator-driven lazy-pages penalty from generated Sails VFT operations and cuts the generated-vs-manual VFT gap by more than 50%.

## Phases

- [x] Confirm thresholds in `CONTROL.md`.
- [x] Refresh baseline under `benchmarks/target/gstd-sails-lazy-pages-overhead/baseline/`.
- [x] Attribute current overhead by row, bucket, section size, and source path.
- [x] Attempt candidate 1: stack/heap message read for generated Sails programs.
- [x] Attempt candidate 2: sync-only dispatch branch.
- [x] Attempt candidate 3: direct hot reply method.
- [x] Run escalation checks for the stack-read survivor.
- [x] Write `RESULTS.md`.
- [x] Run final verification.

## Open Decisions

- Numeric thresholds were approved on 2026-05-20.
- Gear-side lazy-pages changes were not needed for the first win; generated Sails allocation was enough to explain the VFT lazy-pages penalty.
