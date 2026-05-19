# Storage Primitive Selection

Date: 2026-05-19

## Recommendation

Use WAT-shaped static token maps as the primary VFT storage primitive:

- primary wrapper: `sails_storage::gear::StaticVftStorage<BALANCE_LOG2, ALLOWANCE_LOG2>`
- balances: `sails_storage::gear::VftBalances<LOG2_SLOTS>`
- allowances: `sails_storage::gear::VftAllowances<LOG2_SLOTS>`
- active challenger: `sails_storage::gear::StaticMixedActorIdU256Map<LOG2_SLOTS>`
- hot transfer helper: `transfer_actor_u256`
- hot transfer-from helper: `transfer_actor_u256_from`
- layout: `sails_rs::build::StaticMemoryLayout::reserve_actor_u256_map`
  and `reserve_allowance_u256_map`

For 1 million token addresses, `LOG2_SLOTS = 21` gives 2,097,152 slots.
For 2 million token addresses, use `LOG2_SLOTS = 22` to keep load factor away
from the near-full-table regime that made the WAT experiment's 2^20 balance
table slow down near 1M holders. This is the current best hot-path choice for
transfer-shaped workloads. It
avoids allocator growth, keeps key/value layout compact, and performs best in
the million-capacity VFT benchmark.

Keep `FixedOpenAddressMap` and Gear aliases such as `FixedBalanceMap<CAP>` for
small bounded internal state. They are useful and safe, but they are not the
default VFT holder/allowance primitive because the required token capacity is
1-2 million entries.

## Scorecard

Benchmark output:

- `target/storage-primitive-selection/vft-transfer-2026-05-19/bench_data.json`
- `target/storage-primitive-selection/vft-transfer-2026-05-19/vft_million_dynamic_baseline_failed.log`
- `target/storage-primitive-selection/vft-transfer-optimized-2026-05-19/after-combined-transfer-from.json`
- `target/storage-primitive-selection/vft-transfer-optimized-2026-05-19/final-allowance-hash.json`

Commands:

```bash
cargo test --release --manifest-path=benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
cargo test --release --manifest-path=benchmarks/Cargo.toml vft_million_transfer_bench -- --nocapture
```

At 1,000,000 prepared balances and allowances after the optimized WAT-shaped
transfer helpers:

| Backend | Prepare gas | Transfer gas | Transfer-from gas | Recommendation |
| --- | ---: | ---: | ---: | --- |
| `static_balance` + generic allowance | 317,379,116,322,829 | 1,284,751,838 | 1,480,486,416 | Generic fallback |
| `wat_actor_balance` + WAT allowance | 322,774,801,460,398 | 1,252,451,734 | 1,416,295,021 | Default hot path |
| `page_local_actor_balance` + WAT allowance | 321,313,407,834,142 | 1,280,803,990 | 1,467,242,675 | Experimental |
| `control_actor_balance` + WAT allowance | 355,991,311,901,778 | 1,360,221,612 | 1,546,660,297 | Experimental |

`wat_actor_balance` wins both hot transfer rows. In the same optimized run it is
2.51% cheaper than generic static for transfer and 4.34% cheaper for
transfer-from. Generic static still wins prepare by about 2.2%, so it remains
useful for custom key/value tables or workloads where bulk initialization
dominates over steady-state transfers.

## Implemented Optimization

The map now exposes transfer-shaped methods that update storage and return the
new values, avoiding result rereads in the VFT hot path:

- `transfer_actor_u256` validates sender balance and recipient overflow, writes
  both balances, and returns the new balances.
- `transfer_actor_u256_from` validates allowance, sender balance, recipient
  overflow, and capacity before writing allowance and balances.
- The allowance hash uses separate owner/spender multipliers and additive
  mixing, which reduces correlated-pair clustering in the 1M transfer-from
  workload.

Against the original WAT-shaped 1M baseline:

| Operation | Baseline gas | Optimized gas | Delta |
| --- | ---: | ---: | ---: |
| prepare | 324,584,664,276,477 | 322,774,801,460,398 | -0.56% |
| transfer | 1,257,024,975 | 1,252,451,734 | -0.36% |
| transfer-from | 1,446,412,110 | 1,416,295,021 | -2.08% |

An attempted byte-level `U256` arithmetic rewrite regressed transfer rows by
about 0.4-0.5% and was backed out. The stronger signal remains relative to
competing layouts in the same run: WAT-shaped storage is still the best 1M VFT
hot path.

## Dynamic Baselines

The small/medium VFT benchmark compares `BTreeMap`, `HashMap`, `SailsFixed`,
and `SailsStatic` at 16, 64, 128, 256, and 1024 entries. `SailsStatic` starts
winning transfer-shaped hot operations at 256 and is clearly ahead at 1024:

| Operation | Load | Best dynamic baseline | `SailsStatic` delta |
| --- | ---: | --- | ---: |
| transfer | 256 | `BTreeMap` | -16.12% |
| transfer | 1024 | `BTreeMap` | -21.71% |
| transfer-from | 256 | `BTreeMap` | -6.33% |
| transfer-from | 1024 | `BTreeMap` | -14.08% |

It is not a small-map default: at 16-128 entries the dynamic maps can still win,
especially for prepare. That is acceptable because the VFT holder target is
large.

The direct 1M dynamic feasibility run is kept as an ignored/manual benchmark:

```bash
cargo test --release --manifest-path=benchmarks/Cargo.toml vft_million_dynamic_baseline_bench -- --ignored --nocapture
```

The first 1M `BTreeMap` attempt failed during prepare around block 1609, before
static candidates ran. This supports keeping dynamic maps out of the 1-2M VFT
default path.

## Candidate Status

| Candidate | Gas evidence | API safety/usability | Lazy-page shape | Status |
| --- | --- | --- | --- | --- |
| `BTreeMap` / `HashMap` | Best at some small loads; `BTreeMap` failed during direct 1M VFT prepare. | Safe and familiar, but allocator-backed and unbounded. | Heap growth and node/table allocation make page touches less predictable. | Not VFT default. |
| `FixedOpenAddressMap` | Direct lookup wins over `BTreeMap` in aggregator paths, but capacity is compile-time bounded. | Safe, no raw memory, easy to test. | Stored in normal program state, not a large static region. | Bounded internal state only. |
| `StaticOpenAddressTable` / generic static aliases | Best 1M VFT prepare; close to WAT-shaped maps on hot transfer. | Requires generated static layout; `unsafe` only at construction boundary. | Static table with predictable page footprint. | Generic fallback. |
| `StaticActorIdU256Map` + `StaticAllowanceU256Map` | Best 1M transfer and transfer-from rows. | Specialized token API, generated constants, no hand-written addresses. | Compact WAT-shaped slots, no allocator growth. | Recommended VFT hot path. |
| `StaticPageLocalActorIdU256Map` | Very close to WAT-shaped hot rows, not better. | More specialized capacity math. | Page-local control/data hypothesis did not beat WAT-shaped maps. | Experimental. |
| `StaticControlActorIdU256Map` | Loses 1M VFT hot rows and prepare. | More constructor parameters and layout coupling. | Split control/data layout did not pay off here. | Experimental. |
| `StaticGroupedControlActorIdU256Map` | Prior million-entry sweep did not justify promotion. | Most complex layout. | Useful research shape for page-group hypotheses only. | Internal/experimental. |

Recommended:

- `StaticVftStorage<21, 21>` for 1M-core VFT benchmarks.
- `StaticVftStorage<22, 23>` as the starting capacity for 2M holder targets.
- `VftBalances<LOG2>` and `VftAllowances<LOG2>` when separate maps are needed.
- `StaticMemoryLayout` build helpers for reserving memory and generated
  constants.

Current challenger:

- `StaticMixedActorIdU256Map<LOG2>` / `mixed_actor_balance` uses the same
  64-byte layout with an avalanche-mixed actor hash. It now has a positive 1M
  real-cost run versus `wat_actor_balance`, so repeat it and validate at 2M
  before promoting it as the default.

Recommended only for bounded internal state:

- `FixedOpenAddressMap`
- `FixedBalanceMap<CAP>`
- `FixedAllowanceMap<CAP>`

Generic fallback:

- `StaticOpenAddressTable<KEY_SIZE, VALUE_SIZE>` and Gear aliases
  `StaticBalanceTable` / `StaticAllowanceTable`.

Experimental/internal:

- `StaticControlActorIdU256Map`
- `StaticPageLocalActorIdU256Map`
- `StaticGroupedControlActorIdU256Map`

Page-local is very close to WAT-shaped storage in the 1M transfer benchmark but
does not win decisively. Control and grouped layouts should stay research tools
until they show a consistent transfer-shaped win.

## Usage Shape

Build script:

```rust
const TOKEN_LOG2_SLOTS: u8 = 21;

let layout = sails_rs::build::StaticMemoryLayout::new(1024)
    .reserve_actor_u256_map::<TOKEN_LOG2_SLOTS>("balances")
    .reserve_allowance_u256_map::<TOKEN_LOG2_SLOTS>("allowances");

sails_rs::build::build_wasm_with_static_memory(layout);
```

Program code should construct maps from generated constants instead of
hand-written addresses. The only `unsafe` should be isolated at the generated
layout boundary.

Core wrapper:

```rust
let storage = unsafe {
    sails_storage::gear::StaticVftStorage::<TOKEN_LOG2_SLOTS, TOKEN_LOG2_SLOTS>::new(
        static_storage::BALANCES_BASE,
        static_storage::ALLOWANCES_BASE,
    )?
};

let changed = storage.transfer(from, to, amount)?;
```

Static analysis before gas runs:

```bash
python3 scripts/analyze-vft-storage.py \
  --load 1000000 \
  --balance-log2 21 \
  --allowance-log2 21 \
  --actor-hash current \
  --out target/storage-primitive-selection/vft-analysis-2026-05-19/probes-1m-log21.json
```

## Next Work

1. Publish the WAT-shaped token maps and transfer helpers as the VFT
   recommendation in docs/examples.
2. Keep the ignored dynamic 1M benchmark as a feasibility check, not a CI gate.
3. Add a narrower end-to-end VFT benchmark that isolates transfer and
   transfer-from program code so helper code size does not blur per-operation
   deltas.
4. Add an optional dense holder index only if iteration/listing becomes a VFT
   requirement; do not mix it into the transfer hot path.
5. Revisit page-local layout only with a benchmark that proves a consistent
   lazy-page win over WAT-shaped maps.
