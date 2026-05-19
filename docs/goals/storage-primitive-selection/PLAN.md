# Storage Primitive Selection Plan

- [x] Record dirty worktree and current goal controls.
- [x] Inventory current storage primitives and benchmark entrypoints.
- [x] Add or verify a transfer-shaped VFT benchmark.
- [x] Add a million-capacity VFT transfer benchmark.
- [x] Run focused storage feedback checks and save output.
- [x] Compare candidates against `HashMap`/`BTreeMap` and high-capacity static layouts on the scorecard.
- [x] Implement WAT-shaped VFT transfer helpers for 1M-entry balances and allowances.
- [x] Tune the allowance hash for correlated 1M VFT owner/spender pairs.
- [x] Test and reject byte-level `U256` arithmetic when it regressed release gas.
- [x] Rerun the release million-entry VFT benchmark and save optimized results.
- [x] Write `docs/storage-primitive-selection.md`.
- [x] Audit final docs, public/internal status, and benchmark evidence.

Current best candidates:

- Safe bounded state: `FixedOpenAddressMap` and Gear aliases, but not as VFT
  token-address defaults.
- Static token hot path: `StaticActorIdU256Map`/`StaticAllowanceU256Map` for
  1-2 million-entry VFT tables, with `StaticOpenAddressTable` as the generic
  fallback.
- Research-only until proven: control-byte, page-local, and grouped actor maps.

Next command:

```bash
git diff --check
```

Next optimization target:

- Use the WAT-shaped static maps as the default 1M VFT path.
- Add a bool-returning VFT benchmark path next, because real VFT transfer
  returns success/failure, not post-transfer balances.
