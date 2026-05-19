# VFT Storage Better Solution Spec

## Goal

Find and implement a better Gear/Sails VFT storage solution than the current generic `SailsStatic` fast path.

This goal is not complete by showing that old candidates fail. The agent must actively optimize the best current performer, tune its parameters or hot path, or develop a new storage architecture, then prove the result with Gear/Sails gas measurements.

## Baseline Evidence

Use `docs/goals/vft-storage-optimization/RESULTS.md` as phase-1 baseline evidence:

- Current generic `SailsStatic` beats `BTreeMap`, `HashMap`, and `SailsFixed` on the 1024-entry VFT hot path.
- Existing million-entry variants did not beat generic static enough:
  - `MixedActor`: +1.34% weighted
  - `WatActor`: +1.21% weighted
  - `PageLocal`: +0.41% weighted
  - `Control`, `Grouped64`, and `Grouped128`: regressions

That result is a starting point, not a stopping point.

## Required Work

The run must implement or tune at least one real candidate that changes storage behavior, layout, probing, hashing, value handling, or the VFT transfer hot path.

Acceptable directions:

- Optimize current generic `StaticOpenAddressTable` or `StaticBalanceTable`.
- Tune current best performers such as `MixedActor` or `WatActor`.
- Develop a new hybrid layout based on phase/profile evidence.
- Add a specialized VFT balance/allowance primitive if it reduces hot-path gas.
- Change benchmark-only code first, then promote only if evidence is strong.

Unacceptable directions:

- Only rerun existing benchmarks and stop.
- Only document the phase-1 no-winner result.
- Optimize dispatcher/routing/codec instead of storage.
- Publish an API without benchmark evidence.

## Scorecard

Primary metric: weighted median gas across VFT hot operations.

Hot operations:

- `transfer`
- `transfer_fresh`
- `transfer_from`
- `approve`

Target threshold:

- Preferred: at least 7% weighted improvement over current generic `SailsStatic`.
- Acceptable strong narrow win: at least 10% improvement on two hot operations with no hot-op regression above 3%.
- Exploratory progress threshold: at least 3% weighted improvement with a clear path to the 7% target.

If no implemented candidate reaches at least the exploratory threshold, the goal is not done. Pause with evidence and ask for a pivot instead of declaring completion.

## Feedback Loop

Fast check:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Large-capacity check:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture
```

Profile check:

```bash
rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/<label> cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_storage_transfer_bench -- --nocapture
```

## Done When

- A changed or new storage candidate exists in code.
- Benchmarks compare it against current generic `SailsStatic`.
- It reaches the preferred threshold, the strong narrow-win threshold, or at least the exploratory progress threshold.
- Profile artifacts explain why the win exists or where the remaining bottleneck is.
- `RESULTS.md` includes a next-step decision: continue optimizing, promote as experimental, or ask for pivot.
- Focused correctness and benchmark hygiene checks pass.
