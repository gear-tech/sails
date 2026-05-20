# Next Research Plan

## Objective

Lower the remaining generated Sails noop floor without regressing the VFT lazy-pages win. The stack/heap message-read patch fixed the parameter-heavy VFT penalty, but `noop_sails` is still about 142M gas above `noop_gstd`.

## Current Evidence

- `noop_gstd`: 444,865,660 gas.
- `noop_sails`: 587,525,915 gas after the stack/heap read patch.
- `noop_sails` lazy-pages bucket: unchanged at 150,446,687 gas.
- `noop_sails` wasm grew from 26,141 to 26,620 bytes after the patch.
- VFT transfer improved by about 145M gas, mostly from lazy-pages reduction.

## Research Tracks

### 1. Generated Sails Code Shape

Inspect optimized `noop_sails.opt.wasm` for code that should not be required in a sync single-service bool reply:

- panic/error formatting and `unknown_input_panic`
- async reply/signal exports or hooks
- route/interface registry helpers
- metadata or IDL-related code that survives optimization
- generic dispatch branches that remain despite static single-service shape

Acceptance signal: reduce `noop_sails` total gas or wasm code bytes without increasing VFT rows.

### 2. Noop-Specific Minimal Sails Generator Path

Prototype an opt-in generated fast path for simple sync services:

- read header once
- direct match on known `route_id` and `entry_id`
- direct decode for known params
- direct encode/reply for bool, U256, and empty replies
- preserve the Sails Header v1 wire contract

Acceptance signal: close at least 25% of the `noop_sails - noop_gstd` gap while keeping the minimal VFT stack-read improvement.

### 3. Precharge and Section Attribution

Extend the report to separate code-size effects more clearly:

- save per-program optimized wasm before/after
- run `wasm-tools` or equivalent disassembly to identify largest functions
- compare function count and section sizes for `noop_gstd`, `noop_sails`, and `minimal_vft_sails`
- explain how much of `noop_sails` overhead is precharge versus residual Wasm

Acceptance signal: a ranked list of functions/sections with estimated gas impact.

### 4. Gear Lazy-Pages Confirmation

Do not change Gear first. Only inspect Gear lazy-pages if Sails-side experiments stop explaining the cost.

Questions:

- why WAT/gstd noop lazy-pages remains around 140-150M
- whether initial memory setup or post-execution page accounting has fixed cost
- whether code shape changes can avoid touching pages before Gear-side changes are needed

Acceptance signal: evidence that a Gear-side change would affect all rows, not just generated Sails.

## Fast Loop

```bash
SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=target/gstd-sails-lazy-pages-overhead/noop-next \
  rtk cargo test -p benchmarks --features gas-profile --release noop_floor_bench -- --nocapture
```

Run the VFT floor after each candidate that changes generated Sails code:

```bash
SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=target/gstd-sails-lazy-pages-overhead/vft-next \
  rtk cargo test -p benchmarks --features gas-profile --release minimal_vft_floor_bench -- --nocapture
```

## Stop Criteria

Stop when one of these is true:

- `noop_sails - noop_gstd` drops by at least 25%.
- A smaller safe threshold is justified by function-level evidence and no VFT regression.
- Three concrete code-shape candidates fail, and the next work clearly belongs in Gear lazy-pages or precharge accounting.
