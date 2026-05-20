# ATTEMPTS

Record each hypothesis, code change, benchmark run, rejection, and accepted candidate here.

## Baseline

- Artifact root: `benchmarks/target/gstd-sails-lazy-pages-overhead/baseline/`.
- `noop_gstd`: 444,865,660 gas.
- `noop_sails`: 587,088,644 gas.
- `minimal_vft_sails_transfer`: 1,115,816,387 gas.
- `minimal_vft_hot_transfer`: 850,864,350 gas.
- `minimal_vft_sails_transfer` lazy-pages bucket: 588,446,568 gas.

## Candidates

### Candidate 1: Stack/Heap Message Read

- Status: kept.
- Hypothesis: generated Sails `handle` and `init` allocate with `gstd::msg::load_bytes()`, touching extra pages for parameter-heavy VFT calls. Reading through `gstd::msg::with_read_on_stack_or_heap` should avoid the allocator-driven lazy-pages penalty.
- Change: `rs/macros/core/src/program/mod.rs` now reads message payloads through `with_read_on_stack_or_heap`. `rs/src/gstd/macros.rs` clones input only for the async branch because async `message_loop` requires owned input.
- Artifact root: `benchmarks/target/gstd-sails-lazy-pages-overhead/final-stack-read-current/`.
- Result:
  - `minimal_vft_sails_approve`: 953,615,095 -> 808,619,211 gas.
  - `minimal_vft_sails_transfer`: 1,115,816,387 -> 970,818,340 gas.
  - `minimal_vft_sails_transfer_from`: 1,120,830,701 -> 975,485,379 gas.
  - Generated-vs-manual transfer gap: 264,952,037 -> 119,953,990 gas.
  - `minimal_vft_sails_transfer` lazy-pages bucket: 588,446,568 -> 448,588,646 gas.
- Decision: winner for VFT hot paths. The reduction is mostly lazy-pages, not residual Wasm.

### Candidate 2: Sync-Only Dispatch Branch

- Status: rejected.
- Hypothesis: sync-only generated programs could skip the generic asyncness check and async branch.
- Artifact root: `benchmarks/target/gstd-sails-lazy-pages-overhead/candidate-stack-read-sync-dispatch/`.
- Result: no change for `noop_sails` or minimal VFT rows versus Candidate 1; larger VFT framework rows worsened by about 51k gas.
- Decision: reject. LLVM already removes the meaningful sync-only overhead in the small rows, and the extra macro branch does not improve the target score.

### Candidate 3: Direct Hot Reply Method

- Status: rejected and reverted.
- Hypothesis: a generated `try_handle_hot` method that replies directly could avoid the result-handler function-pointer closure.
- Artifact root: `benchmarks/target/gstd-sails-lazy-pages-overhead/candidate-hot-reply/`.
- Result:
  - `noop_sails`: 587,525,915 -> 586,833,331 gas versus Candidate 1, but still only 255k better than baseline.
  - `minimal_vft_sails_transfer`: 970,818,340 -> 971,793,639 gas versus Candidate 1.
  - `minimal_vft_sails` wasm grew from 37,828 to 37,947 bytes.
- Decision: reject. It does not address the real VFT overhead and increases code shape for the main target.
