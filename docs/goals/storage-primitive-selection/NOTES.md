# Storage Primitive Selection Notes

## 2026-05-19 Dirty Baseline

`git status --short` before this goal work showed existing modifications in:

- `Cargo.lock`, `Cargo.toml`
- `benchmarks/Cargo.toml`, `benchmarks/bench_data.json`,
  `benchmarks/build.rs`, `benchmarks/src/benchmarks.rs`,
  `benchmarks/src/clients.rs`, `benchmarks/src/entities.rs`,
  `benchmarks/src/lib.rs`, `benchmarks/storage-stress/Cargo.toml`
- `docs/storage-static-maps-local-notes.md`
- `examples/aggregator/app/Cargo.toml`,
  `examples/aggregator/app/src/lib.rs`,
  `examples/aggregator/app/src/msg_tracker.rs`,
  `examples/aggregator/app/tests/gtest.rs`,
  `examples/aggregator/client/aggregator_client.idl`,
  `examples/aggregator/client/src/aggregator_client.rs`
- `rs/src/build.rs`, `rs/storage/Cargo.toml`, `rs/storage/src/lib.rs`
- untracked `.claude/`, `TODOS.md`,
  `benchmarks/idls/storage_million_program.idl`,
  `benchmarks/src/storage_million_program.rs`,
  `benchmarks/storage-million/`

These are treated as user/work-in-progress changes. Goal edits should be narrow
and avoid reverting them.

## 2026-05-19 Inventory

Current candidates found in `rs/storage` and benchmarks:

- `FixedOpenAddressMap`
- Gear aliases: `FixedBalanceMap`, `FixedAllowanceMap`,
  `StaticBalanceTable`, `StaticAllowanceTable`
- `StaticOpenAddressTable`
- `StaticActorIdU256Map`, `StaticAllowanceU256Map`
- `StaticControlActorIdU256Map`
- `StaticPageLocalActorIdU256Map`
- `StaticGroupedControlActorIdU256Map`

Benchmark entrypoints currently cover isolated storage ops, aggregator tracker
direct lookup/listing, and million-entry static actor maps. A single-message VFT
transfer path is still needed for the final recommendation.

## 2026-05-19 Capacity Correction

VFT token address capacity should be large: target 1-2 million entries. This
means `FixedOpenAddressMap` can only be recommended for bounded internal state,
not as the token-address default.

The benchmark suite now has two transfer-shaped paths:

- `vft_storage_transfer_bench`: compares `BTreeMap`, `HashMap`,
  `SailsFixed`, and `SailsStatic` at small/medium loads.
- `vft_million_transfer_bench`: compares million-capacity static layouts at
  1,000,000 prepared balances and allowances.
- `vft_million_dynamic_baseline_bench`: ignored/manual feasibility check for
  `BTreeMap` and `HashMap` at 1,000,000 prepared balances and allowances.
  The first direct 1M `BTreeMap` run failed during prepare around block 1609,
  so this is not a default green gate.

## 2026-05-19 Implementation Correction

The goal is not only to select a map; it is to develop a better 1M-entry VFT
map path. The current implementation work is therefore centered on
`StaticActorIdU256Map<21>` plus `StaticAllowanceU256Map<21>`.

Implemented hot-path helpers:

- `transfer_actor_u256`: one balance lookup per touched actor, writes both
  balances, returns updated balances.
- `transfer_actor_u256_from`: validates allowance and balances before any
  write, then writes allowance plus balances and returns all updated values.

Saved benchmark artifacts:

- `target/storage-primitive-selection/vft-transfer-optimized-2026-05-19/before.json`
- `target/storage-primitive-selection/vft-transfer-optimized-2026-05-19/after-return-values.json`
- `target/storage-primitive-selection/vft-transfer-optimized-2026-05-19/after-combined-transfer-from.json`
- `target/storage-primitive-selection/vft-transfer-optimized-2026-05-19/final-allowance-hash.json`

## 2026-05-19 WAT Follow-Up

The original `../vara-ft-wat` hash-pair shape is not safe to copy blindly for
the benchmark's correlated `(owner_seed, spender_seed)` pattern. A quick
distribution check at 1,000,000 entries in 2^21 slots showed:

- old rotate/xor combine: average probe `2.50`, max probe `82`;
- literal WAT pair fold: average probe `3272.26`, max probe `26120`;
- two-multiplier additive combine: average probe `1.00`, max probe `1`.

Kept:

- direct 32-bit lane folding with unaligned loads;
- allowance hash as `owner_fold * 0x9E37_79B9 + spender_fold * 0x85EB_CA6B`;
- combined balance/allowance transfer-from helper.

Rejected:

- byte-level `U256` add/sub helpers. They passed correctness tests but
  regressed the release VFT rows, so the code was removed.
