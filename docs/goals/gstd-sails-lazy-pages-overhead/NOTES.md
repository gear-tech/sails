# NOTES

Durable findings for the gstd/Sails/lazy-pages overhead goal.

## Starting Facts

- WAT/gstd lower bound is roughly 437-445M gas in the current benchmark matrix.
- Minimal generated Sails noop is about 142M above `noop_gstd`.
- Current VFT framework noop is about 417M above `noop_gstd`.
- Manual Sails-wire VFT hot path saves about 265-268M gas versus generated Sails minimal VFT for the same static-storage operations.
- Lazy-pages contributes roughly 140-150M in noop floor rows and more in storage-shaped VFT rows.

## Findings

- The VFT gap was not primarily Sails decode or storage map logic. The first large win came from replacing generated `gstd::msg::load_bytes()` reads with `gstd::msg::with_read_on_stack_or_heap()`.
- For generated minimal VFT, the stack/heap read reduced lazy-pages by 139,857,922 gas on approve, transfer, and transfer_from. That lines up with avoiding allocator/page touches during message input handling.
- The pure `noop_sails` row did not improve. It rose by 437,271 gas because the stack/heap read path increased code/precharge slightly while there was no parameter allocation penalty to remove.
- Direct sync dispatch and direct hot reply did not materially help the target. The remaining noop gap is code/precharge/framework shape, not the same lazy-pages problem found in VFT.
- Gear-side lazy-pages changes are not the first production target for this result. Sails codegen can avoid touching those pages for parameter-heavy messages before changing Gear itself.

## Source Pointers

- Sails floor benchmarks: `benchmarks/src/benchmarks.rs`.
- Manual hot path benchmark: `benchmarks/minimal-vft-hot/`.
- Generated minimal VFT: `benchmarks/minimal-vft-sails/`.
- Gear lazy-pages: `../gear/protocol/lazy-pages/`.
