# RESULTS

## Decision

Keep the static variants experimental for now. Do not publish a new default VFT storage primitive from this pass.

The current optimized generic `SailsStatic` fast path is still the best publishable candidate shape, but the evidence does not justify replacing it with WAT/page-local/control/grouped layouts. None of the existing 1M-entry variants met the scorecard threshold of 7% weighted improvement over current static storage.

## Production VFT Baseline

Command:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Result: `1 passed`.

1024-entry medians from `benchmarks/bench_data.json`:

| Backend | transfer | transfer_fresh | transfer_from | approve | Weighted delta vs SailsStatic |
| --- | ---: | ---: | ---: | ---: | ---: |
| BTreeMap | 1530680921 | 1464135857 | 1596648102 | 1274985068 | 22.82% lower for SailsStatic |
| HashMap | 1672979967 | 1497602449 | 1715521410 | 1327391413 | 27.13% lower for SailsStatic |
| SailsFixed | 1358528241 | 1338351274 | 1553173611 | 1118188340 | 15.66% lower for SailsStatic |
| SailsStatic | 1138736114 | 1118559133 | 1306293543 | 964203970 | baseline |

## 1M Candidate Results

Command:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture
```

Result: `1 passed`.

Additional large-capacity storage command:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml storage_million_static_bench -- --nocapture
```

Result: `1 passed`.

Delta versus generic static at 1M entries:

| Candidate | transfer | transfer_fresh | transfer_from | approve | Weighted delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| WatActor | +1.25% | +0.63% | +2.39% | +0.29% | +1.21% |
| MixedActor | +1.25% | +1.14% | +2.40% | +0.29% | +1.34% |
| PageLocal | +0.21% | +0.46% | +0.66% | +0.29% | +0.41% |
| Control | -5.85% | -14.21% | -4.65% | +0.29% | -6.26% |
| Grouped64 | -6.06% | -14.26% | -4.84% | +0.29% | -6.38% |
| Grouped128 | -6.06% | -14.26% | -4.84% | +0.29% | -6.38% |

Positive means lower gas than generic static. None of these clears the 7% weighted threshold.

## Profile Artifacts

Command:

```bash
rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-storage-no-winner-20260519 cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_storage_transfer_bench -- --nocapture
```

Result: `1 passed`.

Artifacts:

- `/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-storage-no-winner-20260519/summary.json`
- `/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-storage-no-winner-20260519/comparison.md`
- `/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/vft-storage-no-winner-20260519/profiles/`

The profile directory contains 524 profile files, including 80 wasm phase JSON files.

## Harness Fix

`benchmarks/src/benchmarks.rs` now avoids writing production `bench_data.json` during `--features gas-profile` runs. The profile run still writes gas-profile artifacts, but no longer pollutes production benchmark medians with profile-shaped wasm totals.

Verified by hashing `benchmarks/bench_data.json` before and after the profile run:

```text
94a16954a52ad30210f52c14009ea8058a9b9a6be060f3f3336831978c068953
```

## Recommendation

Publish path:

- `FixedOpenAddressMap`: safe bounded primitive.
- Current generic static table and VFT fast path: keep as advanced/experimental until API, layout reservation, and migration safety are documented.
- WAT/page-local/control/grouped static variants: keep as research tools only.

Next useful optimization path is not another grouped layout. It is reducing the remaining generic static overhead or improving the production API/build layout story for the current static fast path.

## Final Verification

Passed:

```bash
rtk cargo test -p sails-storage --lib
rtk cargo test --manifest-path benchmarks/Cargo.toml test_data_not_overwritten
rtk cargo fmt --manifest-path benchmarks/Cargo.toml -- --check
rtk git diff --check
```

Observed results:

- `sails-storage`: 50 tests passed.
- `test_data_not_overwritten`: 1 passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
