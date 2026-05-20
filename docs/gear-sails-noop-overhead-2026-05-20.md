# Gear/Sails Noop Overhead Baseline

This note captures the first floor benchmark matrix for reducing fixed Gear/Sails
gas overhead before further storage-map tuning. The profile artifacts were
generated with one release sample per row:

```bash
SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=/tmp/sails-floor-noop-final \
  cargo test -p benchmarks --features gas-profile --release noop_floor_bench -- --nocapture

SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=/tmp/sails-floor-vft-final \
  cargo test -p benchmarks --features gas-profile --release minimal_vft_floor_bench -- --nocapture
```

Use a fresh `SAILS_GAS_PROFILE_DIR` for each comparison. The summary reader
loads every JSON profile under `profiles/`.

## Floor Matrix

| Row | Median gas |
| --- | ---: |
| `noop_wat_sails_wire` | 437,134,646 |
| `noop_wat_raw` | 441,045,976 |
| `noop_gstd` | 444,865,660 |
| `noop_sails` | 587,088,644 |
| `vft_framework_noop` | 862,096,376 |
| `vft_framework_echo_args` | 862,957,777 |

The WAT/gstd lower bound is roughly 437-445M gas. A minimal generated Sails
noop adds about 142M over `noop_gstd`. The existing VFT benchmark noop adds
about 417M over `noop_gstd`, so its floor is dominated by framework/program
shape rather than the storage operation itself.

## Production-Shaped VFT Matrix

| Row | Median gas | Delta vs manual hot path |
| --- | ---: | ---: |
| `minimal_vft_hot_approve` | 688,659,885 | 0 |
| `minimal_vft_sails_approve` | 953,615,095 | 264,955,210 |
| `minimal_vft_hot_transfer` | 850,864,350 | 0 |
| `minimal_vft_sails_transfer` | 1,115,816,387 | 264,952,037 |
| `minimal_vft_hot_transfer_from` | 852,573,595 | 0 |
| `minimal_vft_sails_transfer_from` | 1,120,830,701 | 268,257,106 |

`minimal_vft_hot` is a benchmark-only manual Sails-wire implementation. It is
not a generated Sails API yet. It proves the likely scale of an opt-in hot
dispatch path: about 265-268M gas saved for the same static-storage VFT
operations.

## Wasm Size Drivers

| Program | Bytes | Code | Data |
| --- | ---: | ---: | ---: |
| `noop_wat_raw` | 142 | 19 | 8 |
| `noop_wat_sails_wire` | 229 | 40 | 24 |
| `noop_gstd` | 737 | 590 | 0 |
| `noop_sails` | 26,141 | 21,094 | 4,645 |
| `minimal_vft_sails` | 37,533 | 32,354 | 4,717 |
| `minimal_vft_hot` | 15,875 | 15,520 | 38 |
| `vft_stress` | 74,320 | 68,028 | 5,647 |
| `storage_million` | 121,369 | 111,665 | 8,967 |

This supports two optimization tracks:

1. Keep the production VFT binary close to `minimal_vft_sails`, not the
   multi-backend benchmark binary.
2. Prototype generated hot dispatch because it cuts both code/data size and
   per-message gas in the measured path.

The full generated reports are in `/tmp/sails-floor-noop-final/comparison.md`
and `/tmp/sails-floor-vft-final/comparison.md` on the machine that produced
this baseline.
