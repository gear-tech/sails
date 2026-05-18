# Sails Storage Static Maps - Local Notes

This is a local working note for the storage optimization experiment. It is not
published documentation yet.

## Goal

Explore whether Sails can expose allocator-light storage primitives that make
canonical contracts cheaper, using the `vara-ft-wat` and `awesome-sails` static
VFT examples as the performance reference.

## What Was Implemented

- Added a new `sails-storage` crate under `rs/storage`.
- Added `FixedOpenAddressMap` for bounded, no-allocator map storage.
- Added `StaticOpenAddressTable` for caller-owned static memory regions.
- Added a narrow `sails_rs::build` static-memory build API that emits
  `OUT_DIR/sails_static_storage.rs` and patches the Gear imported memory minimum
  through the existing `gear_wasm_builder::PreProcessor` hook.
- Added Gear-oriented helpers behind the `gear` feature:
  - fixed balance and allowance map aliases
  - static balance and allowance table aliases
  - `ActorId` and `U256` key/value conversion helpers
- Added `benchmarks/storage-stress`, a benchmark-only program comparing:
  - `HashMap`
  - local fixed open-addressed map
  - local raw static map
  - `sails-storage` fixed map
  - `sails-storage` static table backed by generated static memory on WASM
- Wired the storage benchmark into the existing benchmark crate, generated IDL
  and client, and `bench_data.json`.

## Important Correction

The first version had one misleading benchmark label: `SailsStorage` measured the
fixed-map helper aliases, not the static table. This was split into two explicit
backends:

- `SailsFixed` measures `FixedBalanceMap` / `FixedAllowanceMap`.
- `SailsStatic` measures `StaticBalanceTable` / `StaticAllowanceTable`.

The benchmark writer now replaces the whole storage section so old storage keys
do not linger after backend renames.

## Current 1024-Entry Results

Current storage medians in `benchmarks/bench_data.json` show:

| Operation | HashMap | SailsFixed | SailsStatic |
| --- | ---: | ---: | ---: |
| balance insert | 1,137,061,805 | 1,078,413,479 | 924,292,178 |
| balance read | 1,067,742,829 | 850,699,257 | 810,912,211 |
| balance update | 1,142,477,964 | 965,815,430 | 926,264,565 |
| allowance insert | 1,293,302,336 | 1,083,473,052 | 930,873,664 |
| allowance read | 1,221,407,908 | 855,438,899 | 815,651,853 |
| allowance update | 1,300,063,555 | 972,138,658 | 932,579,326 |

Compared with `HashMap`, the bounded forms are about 5-33 percent cheaper for
these generic Sails storage operations. `SailsStatic` is the stronger hot-path
candidate in the refreshed run: at 1024 entries it is about 19-33 percent cheaper
than `HashMap`, and about 5-14 percent cheaper than `SailsFixed`.

The benchmark data now also records preparation cost under `*_prepare_<load>`
keys, so setup/priming gas is visible separately from the operation gas. That is
important for deciding whether a lean path is worth it for repeated hot-state
access rather than one-off state construction.

## Interpretation

`FixedOpenAddressMap` is the safe bounded-map tier:

- no raw memory layout in contract code
- no static page sizing
- easier unit testing
- useful reference implementation for probing and key/value semantics
- good for small bounded state where 16-28 percent over `HashMap` is enough

`StaticOpenAddressTable` is the VFT-style performance tier:

- fixed capacity and explicit storage layout
- intended for hot canonical tables like balances and allowances
- uses explicit static memory sizing from `build.rs`
- best chance to reproduce the WAT-style win when integrated as real lazy-page
  storage, not just as another Sails service benchmark backend

The v1 build API is deliberately narrow:

```rust
let layout = sails_rs::build::StaticMemoryLayout::new(1024)
    .reserve_table::<32, 32>("sails_static_balances", BALANCE_SLOTS)
    .reserve_table::<64, 32>("sails_static_allowances", ALLOWANCE_SLOTS);

sails_rs::build::build_wasm_with_static_memory(layout);
```

It keeps `sails_rs::build_wasm()` unchanged for normal programs, emits generated
constants before Gear's recursive no-build path can return, rejects layouts that
start inside the original imported static pages, and reuses `sails-storage` table
sizing instead of duplicating the slot layout in `sails-rs`.

## Related Awesome-Sails Work

The `awesome-sails` test app now has a static VFT example that uses the static
table primitive. To keep that repo buildable without a sibling checkout, a local
vendored copy of `sails-storage` was added under `awesome-sails/crates` and
marked `publish = false`.

That vendored copy is a temporary bridge. The production path should consume a
released `sails-storage` crate from this Sails repo.

## Verification Run

The following checks passed after the split:

- `cargo test -p sails-storage`
- `cargo test -p sails-storage --features gear`
- `cargo test -p storage-stress`
- `cargo test -p benchmarks test_data_not_overwritten`
- `cargo test --release --manifest-path=benchmarks/Cargo.toml storage_stress_bench -- --nocapture`
- `git diff --check`

For `awesome-sails`, the local verification was:

- `cargo test -p sails-storage --features gear`
- `cargo test -p awesome-sails-test-app static_vft -- --nocapture`
- `cargo test -p awesome-sails-utils`

## Next Work

1. Treat `rs/storage` as the canonical source for the storage primitive.
2. Replace the temporary `awesome-sails` vendored copy with a published or pinned
   dependency once `sails-storage` is ready to publish.
3. Build a canonical VFT storage implementation around `StaticOpenAddressTable`,
   not `HashMap`.
4. Document capacity planning, migration limits, and layout safety for static
   maps before presenting this as production-ready.
