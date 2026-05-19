# VFT Storage Gas Optimization Spec

## Goal

Find, prove, or disprove a storage primitive that lowers Gear/Sails VFT operation gas beyond the current optimized static-table candidate.

The target workload is VFT-style balance and allowance state with large address capacity, ideally 1-2 million entries. The work must stay benchmark-driven and use Gear/Sails gas measurements, not only native Rust timings.

## Non-Goals

- Do not optimize Sails dispatcher, routing, or codec behavior.
- Do not publish a public API until a winning primitive is proven.
- Do not rely only on Rust native microbenchmarks.
- Do not accept improvements that disappear under Gear gas/profile runs.

## Candidate Architectures

- Current static open-address table with tuned probing and load factor.
- Page-local grouped layout to reduce lazy-page touches.
- Split balance and allowance regions optimized for transfer versus approve.
- ActorId-specialized table with cheaper hashing/probing.
- Dense control-page index plus sparse value pages.
- Batch-friendly layout for repeated transfers between nearby accounts.
- Failure-path optimized `transfer` and `transfer_from` that avoid late writes and rereads.

## Scorecard

Primary metric: release-mode median Gear/Sails gas per VFT operation.

Hot operations:

- `transfer` existing recipient
- `transfer` fresh recipient
- `transfer_from`
- `approve`
- optional batch transfer/update paths

Passing threshold:

- Beat `HashMap` and `BTreeMap` by at least 25% on weighted hot-path gas.
- To replace current `SailsStatic`, beat it by either at least 10% on two hot operations or at least 7% weighted total gas with no hot-op regression above 3%.
- If no candidate beats `SailsStatic`, completion requires a clear publish/no-publish decision with evidence that the current static design is near the practical optimum.

## Feedback Loop

Fast check after each meaningful candidate change:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Final checks must include large-capacity and profile runs.

## Done When

- At least three storage candidates beyond current `SailsStatic` are implemented or rejected with code-level rationale.
- Each surviving candidate has release VFT benchmark results against `BTreeMap`, `HashMap`, `SailsFixed`, and current `SailsStatic`.
- At least one large-capacity benchmark targets 1 million entries or documents why the current harness cannot safely run it.
- Gas-profile artifacts are saved under `../gear-dlmalloc/target/storage-primitive-selection/<label>`.
- `RESULTS.md` states whether to publish the current static primitive, publish a new winner, or keep static variants experimental.
- Focused tests and diff hygiene pass.
