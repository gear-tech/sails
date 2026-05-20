# RESULTS

## Decision

Winner: generated Sails programs should read message payloads with
`gstd::msg::with_read_on_stack_or_heap()` instead of unconditionally allocating
with `gstd::msg::load_bytes()`.

This is a productionizable Sails/gstd-side reduction. It does not lower the
pure Sails noop floor, but it materially lowers VFT hot-path gas by avoiding an
allocator-driven lazy-pages penalty on parameter-heavy messages.

## Gas Results

| Row | Baseline | Final | Delta |
| --- | ---: | ---: | ---: |
| `noop_gstd` | 444,865,660 | 444,865,660 | 0 |
| `noop_sails` | 587,088,644 | 587,525,915 | +437,271 |
| `minimal_vft_hot_approve` | 688,659,885 | 688,659,885 | 0 |
| `minimal_vft_sails_approve` | 953,615,095 | 808,619,211 | -144,995,884 |
| `minimal_vft_hot_transfer` | 850,864,350 | 850,864,350 | 0 |
| `minimal_vft_sails_transfer` | 1,115,816,387 | 970,818,340 | -144,998,047 |
| `minimal_vft_hot_transfer_from` | 852,573,595 | 852,573,595 | 0 |
| `minimal_vft_sails_transfer_from` | 1,120,830,701 | 975,485,379 | -145,345,322 |

Generated-vs-manual VFT transfer gap improved from 264,952,037 to 119,953,990
gas, a 54.7% gap reduction.

## Lazy-Pages Deltas

| Row | Baseline lazy-pages | Final lazy-pages | Delta |
| --- | ---: | ---: | ---: |
| `noop_sails` | 150,446,687 | 150,446,687 | 0 |
| `minimal_vft_sails_approve` | 434,081,206 | 294,223,284 | -139,857,922 |
| `minimal_vft_sails_transfer` | 588,446,568 | 448,588,646 | -139,857,922 |
| `minimal_vft_sails_transfer_from` | 588,446,568 | 448,588,646 | -139,857,922 |

The VFT improvement is mostly lazy-pages. Residual Wasm also fell slightly for
transfer, from 227,889,868 to 222,181,551 gas.

## Wasm Size Deltas

| Program | Baseline bytes | Final bytes | Baseline code | Final code |
| --- | ---: | ---: | ---: | ---: |
| `noop_sails` | 26,141 | 26,620 | 21,094 | 21,564 |
| `minimal_vft_sails` | 37,533 | 37,828 | 32,354 | 32,646 |
| `vft_stress` | 74,320 | 74,593 | 68,028 | 68,300 |
| `storage_million` | 121,369 | 121,692 | 111,665 | 111,982 |

The code-size increase explains why `noop_sails` does not improve. The VFT rows
still win because the avoided lazy-pages cost is much larger than the added
precharge/code-size cost.

## Rejected Candidates

- Sync-only dispatch branch: no improvement for target rows and about +51k gas
  on larger VFT framework rows.
- Direct generated hot reply method: `noop_sails` improved only about 255k gas
  versus baseline, while minimal VFT transfer regressed by about 1M gas and wasm
  grew.

## Artifacts

- Baseline noop: `benchmarks/target/gstd-sails-lazy-pages-overhead/baseline/noop/`
- Baseline VFT: `benchmarks/target/gstd-sails-lazy-pages-overhead/baseline/vft/`
- Final noop: `benchmarks/target/gstd-sails-lazy-pages-overhead/final-stack-read-current/noop/`
- Final VFT: `benchmarks/target/gstd-sails-lazy-pages-overhead/final-stack-read-current/vft/`
- Rejected sync dispatch: `benchmarks/target/gstd-sails-lazy-pages-overhead/candidate-stack-read-sync-dispatch/`
- Rejected hot reply: `benchmarks/target/gstd-sails-lazy-pages-overhead/candidate-hot-reply/`

## Files Changed

- `rs/macros/core/src/program/mod.rs`
- `rs/src/gstd/macros.rs`
- `docs/goals/gstd-sails-lazy-pages-overhead/PLAN.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/ATTEMPTS.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/NOTES.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/RESULTS.md`

## Verification

Completed:

- `rtk cargo check -p benchmarks --features gas-profile --tests`
- `SAILS_FLOOR_SAMPLES=5 SAILS_GAS_PROFILE_DIR=target/gstd-sails-lazy-pages-overhead/final-stack-read-current/noop rtk cargo test -p benchmarks --features gas-profile --release noop_floor_bench -- --nocapture`
- `SAILS_FLOOR_SAMPLES=5 SAILS_GAS_PROFILE_DIR=target/gstd-sails-lazy-pages-overhead/final-stack-read-current/vft rtk cargo test -p benchmarks --features gas-profile --release minimal_vft_floor_bench -- --nocapture`
- `rtk cargo fmt -- --check`
- `rtk git diff --check`

## Next Step

Keep this patch for production review. The next noop-floor attack should focus
on generated Sails code/precharge shape, especially panic/error paths, metadata
that leaks into optimized wasm, and whether sync-only programs can avoid unused
exports or framework code without changing the Sails wire contract.
