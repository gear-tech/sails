# PLAN

## Goal

Find and implement a better VFT storage solution than current generic `SailsStatic`.

## Current Strategy

Use phase-1 no-winner results as the baseline, but measure candidate changes side-by-side with the unchanged baseline in the same run. Iteration runs should include only `static_balance` plus the candidate under test; repeated prepares for unchanged backends are not useful until the final matrix run.

Current active result: `inline_owner_account_u256` turns the owner-local allowance architecture into a production-shaped benchmark candidate. It stores the full owner key, balance, and two hot spender allowances in one owner slot, then falls back to an overflow allowance table for third-plus spenders.

## Candidate Backlog

- [x] Inspect prior profile artifacts to identify the largest storage-controlled phase.
- [x] Candidate A: tune generic static transfer bool hot path.
- [x] Candidate B: tune `MixedActor` or `WatActor` where they already show small 1M wins.
- [x] Candidate C: add or prototype a specialized VFT primitive if phase evidence supports it.
- [x] Candidate F: prototype owner-local inline allowances for VFT transfer-from.
- [x] Candidate G: make owner-local inline allowances full-key/full-`U256` with overflow fallback.

## Phases

- [x] Re-establish baseline and active hypothesis.
- [x] Implement/tune candidate A.
- [x] Run fast VFT benchmark.
- [x] Profile promising candidate.
- [x] Run filtered 1M VFT real-cost benchmark.
- [x] Iterate or ask for pivot if no candidate reaches 3% weighted improvement.
- [x] Write `RESULTS.md`.
- [x] Run final verification.

## Open Decisions

- `inline_owner_account_u256` clears exploratory progress at 1M scale and removes the compact-tag shortcut. It is still benchmark-only because it reuses grouped static-memory regions for measurement; public promotion needs explicit static-memory reservation and documented overflow sizing.

## Iteration Command

Use backend filtering while exploring 1M changes. Replace `<candidate>` with one backend such as `inline_owner_account_u256`:

```bash
rtk env SAILS_STORAGE_MILLION_VFT_BACKENDS=static_balance,<candidate> cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture
```

The full backend matrix remains the default when `SAILS_STORAGE_MILLION_VFT_BACKENDS` is unset.
