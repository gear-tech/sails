# RESULTS

## Current Status

Several candidates are implemented and measured. The compact `inline_allowance_balance` prototype proved the architecture, `inline_owner_account_u256` made it full-key/full-`U256`, and the stabilized version now exists as an experimental `sails-storage` API behind `experimental-vft-account`.

Latest same-run 1M real-cost medians:

| Operation | `static_balance` | `inline_owner_account_u256` | Delta |
| --- | ---: | ---: | ---: |
| prepare | 321556277747506 | 211288660377756 | -34.29% |
| transfer | 1478959742 | 1460169545 | -1.27% |
| transfer_fresh | 1474715541 | 1457287604 | -1.18% |
| transfer_from | 1662923723 | 1465702081 | -11.86% |
| approve_fresh | 1303371315 | 1342403479 | +2.99% |
| approve_second | 1305853837 | 1342403479 | +2.80% |
| approve_overflow_third | 1303298666 | 1347703980 | +3.41% |
| transfer_from_overflow | 1665112547 | 1636947889 | -1.69% |

This reaches exploratory progress and a strong narrow win on `transfer_from`, but not preferred 7% weighted success across all hot operations. The primitive is experimental, not a production default. It has a real `StaticMemoryLayout::reserve_vft_account_map` reservation path, but the current 1M benchmark still reuses existing storage-million regions to avoid expanding that crowded benchmark layout.

Profile evidence is saved under `/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-account-static-storage/profiles`.

The profile confirms the mechanism. For `transfer_from`, lazy-page gas drops from `613542773` to `459177411`, because the hot owner balance and hot allowance live in the same owner account slot. Residual Wasm drops from `272518797` to `229662517`. The remaining tradeoff is approve: the public wrapper keeps correctness and overflow behavior, so approve paths are slightly more expensive than the raw benchmark prototype.

Next decision: either build a dedicated focused benchmark crate with non-overlapping account/overflow regions, or continue optimizing `approve` before considering default VFT storage promotion.

## Verification

- `rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture`
- `rtk env SAILS_STORAGE_MILLION_VFT_BACKENDS=static_balance,inline_allowance_balance cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture`
- `rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-storage-better-solution-inline-allowance SAILS_STORAGE_MILLION_VFT_BACKENDS=static_balance,inline_allowance_balance cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_million_real_cost_bench -- --nocapture`
- `rtk env SAILS_STORAGE_MILLION_VFT_BACKENDS=static_balance,inline_owner_account_u256 cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture`
- `rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-account-inline-u256 SAILS_STORAGE_MILLION_VFT_BACKENDS=static_balance,inline_owner_account_u256 cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_million_real_cost_bench -- --nocapture`
- `rtk cargo test -p sails-storage --features experimental-vft-account --lib`
- `rtk cargo test -p sails-rs --features wasm-builder,experimental-vft-account build::`
- `rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-account-static-storage SAILS_STORAGE_MILLION_VFT_BACKENDS=static_balance,inline_owner_account_u256 cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_million_real_cost_bench -- --nocapture`
- `rtk cargo test -p sails-storage --lib`
- `rtk cargo test --manifest-path benchmarks/Cargo.toml test_data_not_overwritten`
- `rtk cargo fmt --manifest-path benchmarks/Cargo.toml -- --check`
- `rtk git diff --check -- benchmarks/src/benchmarks.rs benchmarks/storage-million/src/lib.rs benchmarks/bench_data.json rs/storage/src/lib.rs docs/goals/vft-storage-better-solution`
