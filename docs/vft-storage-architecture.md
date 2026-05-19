# VFT Storage Architecture

Date: 2026-05-19

## Recommendation

Use `sails_storage::gear::StaticVftStorage` as the Rust-facing core VFT map
primitive. It wraps the WAT-shaped static maps:

- `VftBalances<LOG2>`: 64-byte `ActorId + U256` balance slots.
- `VftAllowances<LOG2>`: 96-byte `owner + spender + U256` allowance slots.
- Hot methods: `transfer`, `transfer_from`, `approve`, `mint`, `burn`,
  `balance_of`, and `allowance`.

The active challenger is `StaticMixedActorIdU256Map`, exposed in benchmarks as
`mixed_actor_balance`. It keeps the same 64-byte slot layout but applies an
avalanche-mixed actor hash. In the latest 1M real-cost run it beats the current
`wat_actor_balance` rows, so it should be treated as the leading candidate
pending repeat runs and 2M validation.

For 1 million holders, use at least `LOG2 = 21` for balances. For 2 million
holders, use `LOG2 = 22` to avoid the near-full-table behavior seen in the WAT
experiment. Allowances should usually start at `LOG2 = 23` unless production
data proves allowance cardinality is lower.

## Comparison With Gear Bridges VFT

`../gear-bridges` uses `awesome-sails` VFT storage:

- balances are `ShardedMap<NonZero<ActorId>, NonZero<Balance10>>`;
- allowances are `ShardedMap<(NonZero<ActorId>, NonZero<ActorId>), (NonZero<Allowance9>, u32)>`;
- the service adds expiry updates, minimum-balance rules, pause/storage wrappers,
  event emission, and Sails dispatch/codec overhead.

That design is general and feature-rich, but transfer throughput pays for layers
that are not part of the minimal token hot path.

`../vara-ft-wat` proves the upper bound: a fixed-layout token core with no Rust
runtime, allocator, or generic map. Its gas is much lower, but the implementation
is harder to maintain. `StaticVftStorage` keeps the winning memory shape while
remaining a Rust primitive.

## Current Evidence

The stable benchmark to use is:

```bash
cargo test --release --manifest-path=benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture
```

The current saved 1M random-actor artifact is:

```text
target/storage-primitive-selection/vft-real-cost-2026-05-19/real-cost-random-actors.json
target/storage-primitive-selection/vft-real-cost-2026-05-19/with-mixed-actor-hash.json
```

Best rows from the mixed-hash run:

| Backend | Transfer | Fresh transfer | Transfer-from | Fresh approve |
| --- | ---: | ---: | ---: | ---: |
| `mixed_actor_balance` | 1,296,680,033 | 1,294,205,161 | 1,460,842,155 | 1,134,742,496 |
| `wat_actor_balance` | 1,296,736,962 | 1,300,773,040 | 1,460,899,047 | 1,134,742,496 |
| `static_balance` | 1,313,129,133 | 1,309,080,950 | 1,496,723,850 | 1,138,042,991 |
| `page_local_actor_balance` | 1,310,421,103 | 1,303,117,821 | 1,486,882,004 | 1,134,742,496 |
| `control_actor_balance` | 1,389,883,199 | 1,495,133,064 | 1,566,344,100 | 1,134,742,496 |

The mixed actor hash is the first measured challenger to beat the previous
WAT-shaped actor hash. The transfer/transfer-from wins are small, but prepare
and fresh-recipient transfer improve enough to justify keeping it in the
selection loop. Page-local is close but not better. Control and grouped layouts
add page/control overhead that does not pay off for the current transfer-shaped
workload.

## Non-Wasm Proof Workflow

Use the static/probabilistic analyzer before running expensive Wasm benchmarks:

```bash
python3 scripts/analyze-vft-storage.py \
  --load 1000000 \
  --balance-log2 21 \
  --allowance-log2 21 \
  --actor-hash current \
  --ops 4096 \
  --out target/storage-primitive-selection/vft-analysis-2026-05-19/probes-1m-log21.json
```

Compare the mixed actor-hash candidate before a gas run:

```bash
python3 scripts/analyze-vft-storage.py \
  --load 1000000 \
  --balance-log2 21 \
  --allowance-log2 21 \
  --actor-hash mixed \
  --ops 4096 \
  --out target/storage-primitive-selection/vft-analysis-2026-05-19/probes-1m-log21-mixed.json
```

For WAT/static instruction shape:

```bash
python3 scripts/analyze-vft-storage.py \
  --load 1000000 \
  --balance-log2 21 \
  --allowance-log2 21 \
  --actor-hash current \
  --wat ../vara-ft-wat/wat/extended_vft.wat \
  --out target/storage-primitive-selection/vft-analysis-2026-05-19/wat-audit.json
```

Use this output to reject candidates that increase probe count, page touches, or
hot-path instruction shape before running Gear gas benchmarks.

## Optimization Path

1. Keep `StaticVftStorage` as the primary Rust candidate.
2. Run probe/page analysis for 1M and 2M before changing layout.
3. Run the real-cost benchmark only for candidates that improve the static model.
4. Promote a new layout only if it beats `StaticVftStorage` consistently on
   transfer, transfer-from, and approve.
5. Keep expiry, listing, indexing, and bridge-specific compatibility outside the
   core map until a benchmark proves they belong in the hot path.
