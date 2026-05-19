# NOTES

## Chronological Notes

- 2026-05-19 This goal supersedes the previous no-winner evaluation contract. The previous result remains useful baseline evidence, but cannot complete this goal.
- 2026-05-19 Completion requires an implemented or tuned candidate with measured improvement over current generic `SailsStatic`.
- 2026-05-19 Profile artifacts show `balance_transfer` and `balance_transfer_from` dominate storage-controlled phase cost; result construction is small. First candidate targets bool transfer paths that still use manual get/update or full result construction.
- 2026-05-19 1M VFT iteration should use `SAILS_STORAGE_MILLION_VFT_BACKENDS` to avoid preparing unchanged backends. Example: `static_balance,static_balance_fused,static_balance_fast`.
- 2026-05-19 Candidate A shows validation/result trimming is not enough: same-run 1M real-cost fast path gives about 1-2% on transfer hot paths and no approve gain, below the 3% threshold. Next candidates need a layout or primitive change.
- 2026-05-19 Probe simulation showed current hot transfer seeds are mostly one-probe hits for generic and mixed maps. Collision reduction alone is not enough for the measured real-cost path.
- 2026-05-19 Compact 64-bit tag tables are the strongest direction so far. They reduce transfer-from by about 3.7%, but equal-weighted improvement remains about 2.2% because approve and simple transfers are still dominated by surrounding runtime/Sails costs.
- 2026-05-19 `tag_u64_actor_balance` removes more value-width and construction cost from the benchmark path, but still lands at about 2.23% equal-weighted. This suggests the map representation alone is not yet exposing enough controllable cost in the current real-cost harness.
- 2026-05-19 Fast iteration rule: run only `static_balance,<candidate>` with `SAILS_STORAGE_MILLION_VFT_BACKENDS`; reserve the full backend matrix for final confirmation or regression sweeps.
- 2026-05-19 Capacity-guaranteed compact-tag operations did not move the result beyond about 2.23% equal-weighted. The next useful experiments should change page locality, storage semantics, or the VFT operation shape rather than removing more local branches.
- 2026-05-19 Gas profiles showed compact tags did not change lazy-page gas; all tag wins were residual Wasm. This made page-count reduction the next real lever.
- 2026-05-19 `inline_allowance_balance` validates the owner-local allowance architecture: `transfer_from` now pays the same lazy-page total as a regular transfer because owner balance and allowance are in the same slot.
- 2026-05-19 Production path should be a hybrid: inline 1-2 common spender allowances per owner, overflow table for additional spenders, full `U256` values, and explicit tag collision handling.
- 2026-05-19 `inline_owner_account_u256` validates that the architecture survives full `ActorId` and full `U256` values. The main win remains `transfer_from`; the expected weak spot is third-spender overflow approve, where the owner slot is checked before falling back to the allowance table.
- 2026-05-19 The benchmark implementation reuses the grouped static-memory regions to avoid increasing the static memory layout. That is acceptable for isolated benchmark deployments, but a production primitive needs its own `StaticMemoryLayout` reservation and generated constants.
- 2026-05-19 Stabilization added the real experimental `sails-storage` API and build-layout reservation path. The benchmark still uses storage-million's existing regions, so a dedicated focused benchmark crate remains the clean measurement follow-up.
- 2026-05-19 The library wrapper keeps correctness semantics for inline clears and overflow allowances. That preserves the large `transfer_from` lazy-page win, but approve is now the visible regression boundary.
