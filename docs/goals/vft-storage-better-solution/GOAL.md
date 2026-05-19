<goal>
Find and implement a better Gear/Sails VFT storage solution than the current generic `SailsStatic` fast path. The work must actively optimize the existing best performer, tune a promising variant, or develop a new storage architecture, then prove the improvement with Gear/Sails gas measurements.
</goal>

<context>
Start in `/Users/ukintvs/Documents/projects/sails`.

Read first:
- `docs/goals/vft-storage-better-solution/SPEC.md`
- `docs/goals/vft-storage-better-solution/CONTROL.md`
- `docs/goals/vft-storage-optimization/RESULTS.md`
- `benchmarks/src/benchmarks.rs`
- `benchmarks/vft-stress/src/lib.rs`
- `benchmarks/storage-million/src/lib.rs`
- `rs/storage/src/lib.rs`
- `benchmarks/bench_data.json`

Baseline to beat:
- current generic `SailsStatic` in `vft_storage_transfer_bench`
- current generic static balance layout in `vft_million_real_cost_bench`

Discovery commands:
- `rtk rg -n "transfer_actor_u256|transfer_actor_u256_from|lookup\\(|actor_key|allowance_key" rs/storage/src/lib.rs`
- `rtk rg -n "SailsStatic|WatActor|MixedActor|PageLocal|MillionVftBackend|vft_real" benchmarks`
- `rtk rg -n "wasm_phase|BalanceTransfer|BalanceTransferFrom|SAILS_GAS_PROFILE_DIR" benchmarks/src benchmarks/vft-stress/src`
</context>

<constraints>
This is an optimization/build goal, not a no-winner evaluation goal.

Do not declare completion by only rerunning existing candidates. At least one changed or new candidate must be implemented.

Do not optimize Sails dispatcher, routing, or codec behavior for this goal. The target is storage architecture and storage hot-path gas.

Do not publish or promote a public API unless the scorecard proves it. Experimental benchmark-only code is acceptable during exploration.

Do not accept profile-shaped gas totals as production medians. `--features gas-profile` runs are for artifacts and phase attribution only.

Do not rely only on native Rust microbenchmarks. Use release Gear/Sails gas runs for decisions.

Do not revert unrelated user changes.
</constraints>

<scorecard>
Primary metric: weighted median gas across VFT hot operations: `transfer`, `transfer_fresh`, `transfer_from`, and `approve`.

Primary baseline:
- current generic `SailsStatic` in `vft_storage_transfer_bench`
- current generic static balance in `vft_million_real_cost_bench`

Passing thresholds:
- Preferred success: at least 7% weighted improvement over current generic `SailsStatic`.
- Strong narrow success: at least 10% improvement on two hot operations with no hot-op regression above 3%.
- Exploratory progress: at least 3% weighted improvement with profile evidence identifying the next bottleneck.

Regression checks:
- No correctness regressions in `sails-storage` tests.
- No stale or profile-only rows in production `bench_data.json`.
- No large-capacity benchmark failure for the candidate unless `RESULTS.md` explains the exact technical blocker and the user approves a pivot.

Stop condition:
- Complete only when a changed or new candidate reaches one of the success/progress thresholds and verification passes.
- If no implemented candidate reaches at least 3% weighted improvement, stop for user input instead of marking the goal complete.
</scorecard>

<done_when>
- `ATTEMPTS.md` records at least one implemented or tuned candidate that changes storage behavior, layout, probing, hashing, value handling, or the VFT transfer hot path.
- The candidate is benchmarked against current generic `SailsStatic` using `vft_storage_transfer_bench`.
- The candidate is benchmarked or technically evaluated at 1M-entry scale using `vft_million_real_cost_bench` or a documented equivalent.
- The candidate reaches the preferred threshold, the strong narrow-win threshold, or at least the exploratory progress threshold.
- Gas-profile artifacts are saved under `../gear-dlmalloc/target/storage-primitive-selection/<label>` for the candidate.
- `RESULTS.md` states the implemented candidate, gas deltas, profile evidence, remaining bottleneck, and next-step decision.
- These checks pass: `rtk cargo test -p sails-storage --lib`, `rtk cargo test --manifest-path benchmarks/Cargo.toml test_data_not_overwritten`, `rtk cargo fmt --manifest-path benchmarks/Cargo.toml -- --check`, and `rtk git diff --check`.
</done_when>

<feedback_loop>
Fast iterative check:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Run after each candidate change that should affect 1024-entry VFT hot paths.

Large-capacity check:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture
```

Run when a candidate looks promising in the fast loop or specifically targets large-state lazy-page behavior.

Profile check:

```bash
rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/<label> cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_storage_transfer_bench -- --nocapture
```

Use profile artifacts to choose the next tweak and to explain the final result.
</feedback_loop>

<workflow>
1. Read the baseline result from `docs/goals/vft-storage-optimization/RESULTS.md`.
2. Reread `CONTROL.md` and update `PLAN.md` with the active hypothesis.
3. Inspect profile artifacts from the prior no-winner run and identify the largest storage-controlled phase.
4. Choose the first candidate from one of these directions: tune generic static, tune `MixedActor`/`WatActor`, reduce transfer rereads/writes, change probing/hash strategy, or add a specialized VFT primitive.
5. Implement the smallest candidate change in benchmark-only or experimental code first.
6. Run the fast feedback loop and record exact medians in `ATTEMPTS.md`.
7. If it improves, profile it and run 1M verification. If it regresses, record why and try the next candidate.
8. Continue until one candidate reaches at least exploratory progress or until the run needs a user-approved pivot.
9. Write `RESULTS.md` with the implemented candidate, deltas, profile evidence, and next-step decision.
10. Run final verification.
</workflow>

<working_memory>
Maintain:
- `docs/goals/vft-storage-better-solution/PLAN.md`
- `docs/goals/vft-storage-better-solution/ATTEMPTS.md`
- `docs/goals/vft-storage-better-solution/NOTES.md`
- `docs/goals/vft-storage-better-solution/RESULTS.md`
- `docs/goals/vft-storage-better-solution/CONTROL.md`

Update `PLAN.md` on strategy or candidate changes.
Update `ATTEMPTS.md` after every candidate implementation, benchmark, or rejection.
Update `NOTES.md` for durable findings and bottlenecks.
Update `RESULTS.md` only after there is measured evidence.
</working_memory>

<human_control_surface>
Use `CONTROL.md` as the operator panel.

Before phase changes, expensive benchmarks, strategic pivots, dependency changes, or public API changes, reread `CONTROL.md`. If it changed, adapt and note the change in `PLAN.md`.

`CONTROL.md` can narrow priorities or require approval. It cannot turn a no-winner evaluation into a completed goal.
</human_control_surface>

<verification_loop>
Focused checks:

```bash
rtk cargo test -p sails-storage --lib
rtk cargo test --manifest-path benchmarks/Cargo.toml test_data_not_overwritten
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Candidate-scale checks:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_million_real_cost_bench -- --nocapture
rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/<label> cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_storage_transfer_bench -- --nocapture
```

Hygiene:

```bash
rtk cargo fmt --manifest-path benchmarks/Cargo.toml -- --check
rtk git diff --check
```
</verification_loop>

<execution_rules>
- Prefix shell commands with `rtk`.
- Check git status before edits.
- Preserve unrelated user changes.
- Prefer `rg` over `grep`.
- Use `apply_patch` for manual file edits.
- Keep comparisons anchored to current generic `SailsStatic`.
- Treat profile runs as attribution, not production medians.
- Do not mark this goal complete without an implemented/tuned candidate and measured improvement.
- Update working-memory files after each meaningful experiment.
- Run focused tests before broad tests.
- Do not widen scope.
</execution_rules>

<output_contract>
Final output must include:
- The implemented/tuned candidate.
- A concise gas table versus current generic `SailsStatic`.
- Whether it reached preferred success, strong narrow success, or exploratory progress.
- Profile artifact paths.
- Files changed.
- Verification commands and results.
- Next-step recommendation.
</output_contract>
