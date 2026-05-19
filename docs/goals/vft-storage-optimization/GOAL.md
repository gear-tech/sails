<goal>
Explore, implement, and measure storage architectures that reduce Gear/Sails VFT operation gas versus `BTreeMap`, `HashMap`, `FixedOpenAddressMap`, and the current optimized `StaticOpenAddressTable` fast path. The target is large VFT state with 1-2 million balances/allowances, transfer-shaped workloads, and Gear lazy-pages cost sensitivity.
</goal>

<context>
Start in `/Users/ukintvs/Documents/projects/sails`.

Read these files first:
- `docs/goals/vft-storage-optimization/SPEC.md`
- `docs/goals/vft-storage-optimization/CONTROL.md`
- `benchmarks/src/benchmarks.rs`
- `benchmarks/vft-stress/src/lib.rs`
- `benchmarks/storage-million/src/lib.rs`
- `rs/storage/src/lib.rs`
- `benchmarks/bench_data.json`

Useful sibling context:
- `/Users/ukintvs/Documents/projects/gear-dlmalloc/docs/benchmarks/`
- `/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/`
- `/Users/ukintvs/Documents/projects/gear/protocol/lazy-pages/`

Discovery commands:
- `rtk rg -n "VftStorageBackend|SailsStatic|StaticOpenAddress|transfer_actor_u256|storage_million" benchmarks rs/storage/src`
- `rtk rg -n "SAILS_GAS_PROFILE_DIR|write_gas_profile|gas-profile" benchmarks/src`
</context>

<constraints>
Do not optimize Sails dispatcher, routing, or codec behavior for this goal. The target is storage architecture and storage hot-path gas.

Do not publish or promote a public API unless the scorecard proves it. Experimental variants may stay benchmark-only.

Do not rely only on native Rust microbenchmarks. Use Gear/Sails release gas runs for decisions.

Do not accept a candidate whose win disappears under `--features gas-profile` or large-capacity checks.

Do not revert unrelated user changes. This workspace may already be dirty from benchmark work.

Do not widen the scope into allocator internals unless the storage evidence specifically points to allocator behavior as the limiting cost.
</constraints>

<scorecard>
Primary metric: median release-mode Gear/Sails gas per VFT hot operation.

Hot operations:
- `transfer` existing recipient
- `transfer` fresh recipient
- `transfer_from`
- `approve`
- optional batch transfer/update paths when the candidate is batch-oriented

Passing threshold:
- Candidate must beat `HashMap` and `BTreeMap` by at least 25% on weighted hot-path gas.
- Candidate replaces current `SailsStatic` only if it beats it by at least 10% on two hot operations, or by at least 7% weighted total gas with no hot-op regression above 3%.
- If no candidate beats current `SailsStatic`, stop only after `RESULTS.md` explains why the current static design should be published, kept experimental, or deferred.

Regression checks:
- No correctness regressions in `sails-storage` tests.
- No stale or profile-only rows in production `bench_data.json`.
- No unexplained large-capacity failure.

Scoring command or inspection path:
- Fast score: `benchmarks/bench_data.json` after `vft_storage_transfer_bench`.
- Profile score: `../gear-dlmalloc/target/storage-primitive-selection/<label>/summary.json` and `profiles/*_wasm_phases.json`.
- Large score: `storage_million_static_bench` output and recorded benchmark data.

Stop condition:
- A candidate meets the replacement threshold and passes final verification, or at least three credible candidates fail the threshold and `RESULTS.md` documents the decision.
</scorecard>

<done_when>
- `docs/goals/vft-storage-optimization/ATTEMPTS.md` records at least three storage candidates beyond current `SailsStatic`, either implemented and measured or rejected with code-level rationale.
- Each surviving candidate has release VFT benchmark results against `BTreeMap`, `HashMap`, `SailsFixed`, and current `SailsStatic`.
- At least one large-capacity benchmark targets 1 million entries, or `RESULTS.md` documents the exact technical blocker preventing that run.
- Gas-profile artifacts are saved under `../gear-dlmalloc/target/storage-primitive-selection/<label>` for the final candidate or final no-winner comparison.
- `docs/goals/vft-storage-optimization/RESULTS.md` states one of: publish current static primitive, publish a new winning primitive, or keep all static variants experimental.
- These checks pass: `rtk cargo test -p sails-storage --lib`, `rtk cargo test --manifest-path benchmarks/Cargo.toml test_data_not_overwritten`, and `rtk git diff --check`.
</done_when>

<feedback_loop>
Fast iterative check:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Expected runtime: minutes after the first compile. Run it after each meaningful candidate implementation or tuning pass.

Proxy validity: this is the fastest representative Gear/Sails VFT transfer workload and catches obvious storage losers before expensive large-capacity runs.

Escalation checks:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml storage_million_static_bench -- --nocapture
rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/<label> cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_storage_transfer_bench -- --nocapture
```

Use escalation when a candidate looks competitive in the fast loop or when deciding to stop.
</feedback_loop>

<workflow>
1. Inspect `SPEC.md`, `CONTROL.md`, current benchmark code, and current storage primitives.
2. Update `PLAN.md` with the active phase and candidate list.
3. Establish the current `SailsStatic` baseline from the latest production VFT benchmark data.
4. Pick one candidate at a time. Prefer small, measurable storage-layout or hot-path changes.
5. Implement the candidate in a benchmark-only or clearly experimental surface first.
6. Run the fast feedback check and record medians in `ATTEMPTS.md`.
7. Kill candidates that fail the threshold or add complexity without a plausible gas win.
8. For survivors, run large-capacity and gas-profile escalation checks.
9. Write `RESULTS.md` with the final recommendation and publishability decision.
10. Run final verification and leave the working tree in an inspectable state.
</workflow>

<working_memory>
Maintain these files throughout the run:
- `docs/goals/vft-storage-optimization/PLAN.md`
- `docs/goals/vft-storage-optimization/ATTEMPTS.md`
- `docs/goals/vft-storage-optimization/NOTES.md`
- `docs/goals/vft-storage-optimization/RESULTS.md`
- `docs/goals/vft-storage-optimization/CONTROL.md`

Update `PLAN.md` when the phase, active candidate, or strategy changes.

Update `ATTEMPTS.md` after each implementation attempt, failed experiment, benchmark run, or successful scoring improvement.

Update `NOTES.md` when durable context, constraints, blockers, or surprising results are discovered.

Update `RESULTS.md` only when evidence is strong enough to support a candidate decision or final recommendation.
</working_memory>

<human_control_surface>
Use `docs/goals/vft-storage-optimization/CONTROL.md` as the compact human operator panel.

Before each phase change, strategic pivot, expensive benchmark, dependency change, or public API change, reread `CONTROL.md`. If it changed, summarize the relevant change in `PLAN.md` and adapt before proceeding.

`CONTROL.md` may narrow priorities, adjust thresholds, pause work, or require approval. It must not silently weaken this `GOAL.md` scorecard or `done_when`.
</human_control_surface>

<verification_loop>
Focused checks:

```bash
rtk cargo test -p sails-storage --lib
rtk cargo test --manifest-path benchmarks/Cargo.toml test_data_not_overwritten
rtk cargo test --release --manifest-path benchmarks/Cargo.toml vft_storage_transfer_bench -- --nocapture
```

Profile and large-capacity checks:

```bash
rtk cargo test --release --manifest-path benchmarks/Cargo.toml storage_million_static_bench -- --nocapture
rtk env SAILS_GAS_PROFILE_DIR=/Users/ukintvs/Documents/projects/gear-dlmalloc/target/storage-primitive-selection/<label> cargo test --release --manifest-path benchmarks/Cargo.toml --features gas-profile vft_storage_transfer_bench -- --nocapture
```

Hygiene:

```bash
rtk cargo fmt --manifest-path benchmarks/Cargo.toml -- --check
rtk git diff --check
```

If a check cannot run, record the exact command, failure, and impact in `ATTEMPTS.md` or `RESULTS.md`.
</verification_loop>

<execution_rules>
- Prefix shell commands with `rtk`.
- Check git status before edits.
- Preserve unrelated user changes.
- Prefer `rg` over `grep` when available.
- Use `apply_patch` for manual file edits.
- Read context files before implementation.
- Batch independent file reads in parallel when possible.
- Keep the scorecard current and compare against current `SailsStatic`, not only against `HashMap`.
- Use the fastest representative feedback check while iterating; reserve slower checks for escalation and final verification.
- Maintain `PLAN.md`, `ATTEMPTS.md`, `NOTES.md`, `RESULTS.md`, and `CONTROL.md`.
- Update `ATTEMPTS.md` after each meaningful candidate or benchmark result.
- Run focused tests before broad tests.
- Do not paper over failures.
- Do not widen scope.
- Keep the final answer concise.
</execution_rules>

<output_contract>
Final output must include:
- The winning candidate or explicit no-winner decision.
- A short table of median gas deltas versus `BTreeMap`, `HashMap`, `SailsFixed`, and current `SailsStatic`.
- Paths to saved benchmark/profile artifacts.
- Files changed.
- Verification commands run and their results.
- A publishability recommendation: production-ready, experimental, or do not publish.
</output_contract>
