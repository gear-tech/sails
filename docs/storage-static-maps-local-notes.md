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
- Added Gear-native helpers in the default `sails-storage` surface:
  - fixed balance and allowance map aliases
  - static balance and allowance table aliases
  - `ActorId` and `U256` key/value conversion helpers
  - actor-specific WAT-shaped, separated-control, and page-local static maps
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

## Aggregator Example Tracker Results

The `examples/aggregator` message tracker now has two selectable backends:

- `BTree`, the original `BTreeMap<MessageId, OpStatus>` storage.
- `SailsFixed`, a `sails-storage::FixedOpenAddressMap<32, 1, 2048>` storage.

The benchmark harness records the example-shaped tracker workload under
`examples.aggregator_*` keys in `benchmarks/bench_data.json`.

Current 1024-entry medians show:

| Operation | BTree | SailsFixed | Change |
| --- | ---: | ---: | ---: |
| prepare | 12,509,943,267 | 8,915,089,901 | -28.7% |
| insert fresh | 1,749,165,135 | 1,371,924,001 | -21.6% |
| update existing | 1,767,823,396 | 1,379,752,436 | -22.0% |
| read existing | 1,653,309,810 | 1,259,882,121 | -23.8% |
| list statuses | 2,305,857,784 | 24,785,539,631 | +974.9% |

Interpretation: the fixed map is a real win for hot point operations in an
example state path, but it is the wrong backing store for list-heavy APIs unless
the program keeps a separate compact index of visible keys. The current fixed
map iterator scans the configured capacity, so full listing pays for empty
slots.

## Million-Entry Static Table Check

The benchmark suite now includes `benchmarks/storage-million`, a focused
static-table program that fills one balance table to 1,000,000 visible entries.
The generic, WAT-shaped, and separated-control backends reserve 2,097,152 slots;
the page-local and grouped-control backends reserve 8,192 Gear pages for
2,064,384 slots. The measured point operations therefore run at about 48 percent
occupancy instead of treating the final five percent of an open-addressed table
as the normal case.

Preparation is split into 512-entry messages because gtest already uses the
maximum user gas limit and large random-probe chunks hit too many fresh static
pages in one block.

Current medians in `benchmarks/bench_data.json`:

| Operation | Generic static | WAT actor static | Control actor static | Page-local actor static | Page-local vs generic |
| --- | ---: | ---: | ---: | ---: | ---: |
| prepare total | 153,756,361,755,267 | 162,032,206,944,470 | 195,256,963,709,380 | 160,579,059,641,744 | +4.4% |
| insert fresh | 905,528,184 | 903,896,690 | 1,053,190,461 | 902,761,215 | -0.3% |
| update existing | 906,132,546 | 905,939,676 | 945,833,568 | 906,080,283 | -0.0% |
| read existing | 790,205,331 | 789,947,148 | 829,878,697 | 790,118,133 | -0.0% |
| read missing | 789,394,367 | 787,600,137 | 786,470,759 | 786,477,580 | -0.4% |
| remove | 904,825,969 | 905,734,298 | 944,623,965 | 904,864,645 | +0.0% |

The same benchmark also records 512-op batches inside one Sails call to reduce
fixed dispatch/decode/reply noise:

| Batch operation | Generic static | WAT actor static | Control actor static | Page-local actor static | Page-local vs generic |
| --- | ---: | ---: | ---: | ---: | ---: |
| insert fresh x512 | 79,973,731,991 | 83,432,730,040 | 100,178,677,973 | 82,436,303,524 | +3.1% |
| update existing x512 | 83,165,179,044 | 83,992,895,596 | 89,141,972,716 | 84,064,886,380 | +1.1% |
| read existing x512 | 23,344,239,101 | 24,604,721,260 | 29,773,078,764 | 24,692,265,580 | +5.8% |
| read missing x512 | 22,964,573,027 | 23,403,071,904 | 7,548,077,598 | 22,989,651,275 | +0.1% |
| remove x512 | 79,872,000,466 | 83,887,742,060 | 44,522,952,556 | 83,442,479,724 | +4.5% |

The grouped-control sweep tested multi-page control/data groups between dense
global control and one-page-local control:

| Backend | Prepare | Read existing x512 | Read missing x512 | Remove x512 | Hit vs static | Missing vs static | Remove vs static |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Control | 195,256,963,709,380 | 29,773,078,764 | 7,548,077,598 | 44,522,952,556 | +27.5% | -67.1% | -44.3% |
| Page-local | 160,579,059,641,744 | 24,692,265,580 | 22,989,651,275 | 83,442,479,724 | +5.8% | +0.1% | +4.5% |
| Grouped pages2 | 237,753,706,205,388 | 35,618,520,743 | 23,136,505,737 | 93,977,463,259 | +52.6% | +0.7% | +17.7% |
| Grouped pages4 | 275,962,049,516,030 | 40,670,936,470 | 23,640,347,798 | 99,308,358,593 | +74.2% | +2.9% | +24.3% |
| Grouped pages8 | 295,066,822,958,099 | 43,217,035,734 | 23,355,072,897 | 101,973,806,260 | +85.1% | +1.7% | +27.7% |
| Grouped pages16 | 287,183,578,573,915 | 42,182,682,908 | 20,911,649,220 | 94,214,099,294 | +80.7% | -8.9% | +18.0% |
| Grouped pages32 | 232,874,064,951,030 | 34,902,430,325 | 12,920,036,326 | 64,326,065,235 | +49.5% | -43.7% | -19.5% |
| Grouped pages64 | 197,165,292,858,774 | 30,088,711,404 | 16,117,385,384 | 44,925,344,108 | +28.9% | -29.8% | -43.8% |
| Grouped pages128 | 197,395,177,029,018 | 30,088,711,404 | 22,533,559,584 | 44,925,344,108 | +28.9% | -1.9% | -43.8% |

Interpretation: the static table still has cheap point operations after one
million entries, but construction cost is real and must be amortized. The
WAT-shaped actor map does not currently beat the generic static table in this
Sails benchmark. The page-local actor map validates the locality hypothesis only
partly: it removes most of the separated-control table's hit-path penalty by
keeping control bytes and data slots in the same Gear page, but it does not keep
the separated-control table's missing-read and remove wins. The grouped-control
sweep did not find a useful middle point. `pages32` gets the best missing-read
result among grouped layouts but still makes read-existing batches 49.5 percent
more expensive than generic static. `pages64` recovers remove cost but misses the
missing-read target and still carries a 28.9 percent hit-path penalty. Keep these
layouts as negative benchmark evidence, not storage recommendations.

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
- specialized WAT-shaped, separated-control, page-local, grouped-control actor,
  and WAT-shaped allowance maps are now available, but the first million-entry
  actor benchmark did not show a useful default replacement for the generic
  static table

The v1 build API is deliberately narrow:

```rust
let layout = sails_rs::build::StaticMemoryLayout::new(1024)
    .reserve_table::<32, 32>("sails_static_balances", BALANCE_SLOTS)
    .reserve_table::<64, 32>("sails_static_allowances", ALLOWANCE_SLOTS)
    .reserve_actor_u256_map::<21>("wat_actor_balances")
    .reserve_control_actor_u256_map::<21>("control_actor_balances")
    .reserve_page_local_actor_u256_map::<13>("page_local_actor_balances")
    .reserve_grouped_control_actor_u256_map::<8, 5>("grouped_actor_balances_pages32")
    .reserve_allowance_u256_map::<21>("wat_allowances");

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
- `cargo test -p sails-storage`
- `cargo test -p sails-rs --features wasm-builder build::tests`
- `cargo test -p storage-stress`
- `cargo test -p storage-million`
- `cargo test -p benchmarks test_data_not_overwritten`
- `cargo test -p benchmarks --bin bench-analyzer`
- `cargo test --release --manifest-path=benchmarks/Cargo.toml storage_stress_bench -- --nocapture`
- `cargo test --release --manifest-path=benchmarks/Cargo.toml storage_million_static_bench -- --nocapture`
- `git diff --check`

For `awesome-sails`, the local verification was:

- `cargo test -p sails-storage`
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
