<goal>
Investigate and reduce fixed Gear/gstd/Sails execution overhead that dominates noop and VFT hot-path gas before storage-map cost becomes visible. Produce measured changes or a no-winner report across raw WAT, minimal gstd, generated Sails, production-shaped Sails VFT, manual Sails-wire hot path, and Gear lazy-pages buckets.
</goal>

<context>
Start in `/Users/ukintvs/Documents/projects/sails`.

Read these files first:
- `docs/goals/gstd-sails-lazy-pages-overhead/SPEC.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/CONTROL.md`
- `docs/gear-sails-noop-overhead-2026-05-20.md`
- `benchmarks/src/benchmarks.rs`
- `benchmarks/noop-wat/`
- `benchmarks/noop-gstd/`
- `benchmarks/noop-sails/`
- `benchmarks/minimal-vft-sails/`
- `benchmarks/minimal-vft-hot/`
- `rs/src/`
- `rs/macros/`

Useful sibling context:
- `/Users/ukintvs/Documents/projects/gear/protocol/lazy-pages/`
- `/Users/ukintvs/Documents/projects/gear/ethexe/processor/src/host/api/lazy_pages.rs`
- `/Users/ukintvs/Documents/projects/gear-dlmalloc/docs/benchmarks/`

Discovery commands:
- `rtk rg -n "noop_floor_bench|minimal_vft_floor_bench|SAILS_GAS_PROFILE_DIR|Residual Wasm|lazy" benchmarks/src`
- `rtk rg -n "program|service|route|dispatch|reply|encode|decode" rs/src rs/macros`
- `rtk rg -n "process_lazy_pages|write_accessed_pages|pre_process_memory_accesses|GearPage" ../gear/protocol/lazy-pages ../gear/ethexe/processor/src/host/api`
</context>

<constraints>
Use release-mode Gear/Sails benchmarks for decisions. Native microbenchmarks may explain a hypothesis but cannot be the final score.

Preserve the Sails wire contract unless a change is explicitly benchmark-only. A manual hot path is evidence, not a production API by itself.

Do not resume storage-map architecture work unless the overhead attribution proves the map interacts with lazy-pages, code size, or precharge in a way that changes the floor.

Do not add broad dispatcher instrumentation. Prefer the existing gas-profile buckets, section-size reports, focused benchmark variants, and narrowly scoped phase markers only when needed.

Do not treat WAT as the production target. WAT remains the lower-bound reference.

Do not revert unrelated user changes. This workspace may already be dirty from benchmark and storage work.
</constraints>

<scorecard>
Primary metric: median release-mode gas in `noop_floor_bench` and `minimal_vft_floor_bench`.

Baseline rows:
- `noop_wat_raw`: 441,045,976
- `noop_gstd`: 444,865,660
- `noop_sails`: 587,088,644
- `vft_framework_noop`: 862,096,376
- `minimal_vft_hot_transfer`: 850,864,350
- `minimal_vft_sails_transfer`: 1,115,816,387

Passing thresholds:
- Reduce `noop_sails - noop_gstd` by at least 25%, or prove with source-level evidence why this repo cannot reduce it.
- Reduce `minimal_vft_sails_* - minimal_vft_hot_*` by at least 25% for two hot VFT operations, or land enough generated hot-path infrastructure that the remaining gap is clearly bounded by measured follow-up work.
- No optimized lower-bound row may regress `noop_gstd` or raw WAT by more than 3%.
- Report Wasm total/code/data section deltas for every candidate.
- Report lazy-pages bucket deltas for noop and VFT rows when `gas-profile` data is available.

Regression checks:
- Existing floor benchmark rows still run.
- Existing generated Sails clients and IDLs remain consistent.
- No production `bench_data.json` corruption or stale rows are introduced.
- Storage benchmark correctness tests still pass if storage crates are touched.

Scoring command or inspection path:
- `/tmp/sails-floor-<label>/comparison.md` or a repo/target artifact path selected in `CONTROL.md`.
- `docs/goals/gstd-sails-lazy-pages-overhead/ATTEMPTS.md`.
- `docs/goals/gstd-sails-lazy-pages-overhead/RESULTS.md`.

Stop condition: one validated reduction meets the threshold and passes final verification, or at least three credible reduction candidates fail and `RESULTS.md` explains the next best architecture choice.
</scorecard>

<done_when>
- `docs/goals/gstd-sails-lazy-pages-overhead/ATTEMPTS.md` records at least three candidates with measurements or code-level rejection rationale.
- Fresh baseline and final comparison reports exist under `target/gstd-sails-lazy-pages-overhead/<label>/` or another path recorded in `CONTROL.md`.
- `RESULTS.md` includes before/after gas, Sails/gstd deltas, VFT hot-path deltas, Wasm section-size deltas, lazy-pages bucket deltas, and final recommendation.
- If code changes are made, focused tests and benchmarks pass, including `noop_floor_bench` and `minimal_vft_floor_bench`.
- These checks pass unless documented as out of scope with exact failure output: `rtk cargo fmt -- --check`, `rtk cargo check -p benchmarks --features gas-profile --tests`, and `rtk git diff --check`.
</done_when>

<feedback_loop>
Fast iterative check:

```bash
rtk env SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=/tmp/sails-floor-fast cargo test -p benchmarks --features gas-profile --release noop_floor_bench -- --nocapture
```

Expected runtime: under a minute after compile. Run after each noop or code-size candidate.

VFT hot-path check:

```bash
rtk env SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=/tmp/sails-floor-vft cargo test -p benchmarks --features gas-profile --release minimal_vft_floor_bench -- --nocapture
```

Expected runtime: minutes after compile. Run after each Sails hot-dispatch or VFT-shape candidate.

Proxy validity: these are the smallest representative rows that expose raw WAT, gstd, generated Sails, manual hot path, code size, precharge, and lazy-pages buckets.

Slower escalation check: run both checks with more samples, save artifacts under `target/gstd-sails-lazy-pages-overhead/<candidate>/`, and compare medians plus section-size reports before accepting a candidate.
</feedback_loop>

<workflow>
1. Inspect `SPEC.md`, `CONTROL.md`, current floor report, benchmark crates, Sails macro/runtime code, and Gear lazy-pages files.
2. Update `PLAN.md` with the active phase, artifact path, and candidate list.
3. Refresh the baseline using the current tree and save it under the artifact path.
4. Attribute current overhead by comparing WAT, gstd, generated Sails, manual hot path, Wasm section sizes, precharge buckets, lazy-pages buckets, and residual Wasm.
5. Pick one candidate at a time from Sails hot dispatch, Sails/gstd size reduction, lazy-pages reduction, or precharge reduction.
6. Implement the smallest measurable change. Keep benchmark-only experiments clearly marked.
7. Run the fast loop, record results in `ATTEMPTS.md`, and kill candidates that miss the scorecard without a plausible next step.
8. For survivors, run VFT and multi-sample escalation checks and record section-size and bucket deltas.
9. Write `RESULTS.md` with the final recommendation and whether follow-up work belongs in Sails, gstd, Gear lazy-pages, or benchmark-only research.
10. Run final verification and leave the working tree inspectable.
</workflow>

<working_memory>
Maintain these files throughout the run:
- `docs/goals/gstd-sails-lazy-pages-overhead/PLAN.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/ATTEMPTS.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/NOTES.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/RESULTS.md`
- `docs/goals/gstd-sails-lazy-pages-overhead/CONTROL.md`

Update `PLAN.md` when phase, active candidate, artifact path, or strategy changes.

Update `ATTEMPTS.md` after each hypothesis, code change, benchmark run, rejection, or accepted candidate.

Update `NOTES.md` for durable source-level findings, lazy-pages behavior, precharge explanations, blockers, and assumptions.

Update `RESULTS.md` only when evidence supports a candidate decision or final recommendation.
</working_memory>

<human_control_surface>
Use `docs/goals/gstd-sails-lazy-pages-overhead/CONTROL.md` as the compact operator panel.

Before each phase change, strategic pivot, expensive benchmark, Gear-side change, public API change, or dependency change, reread `CONTROL.md`. If it changed, summarize the relevant change in `PLAN.md` and adapt before proceeding.

`CONTROL.md` may narrow priorities, adjust thresholds, pause work, or require approval. It must not silently weaken this `GOAL.md` scorecard or `done_when`.
</human_control_surface>

<verification_loop>
Focused checks:

```bash
rtk cargo check -p benchmarks --features gas-profile --tests
rtk env SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=/tmp/sails-floor-fast cargo test -p benchmarks --features gas-profile --release noop_floor_bench -- --nocapture
rtk env SAILS_FLOOR_SAMPLES=1 SAILS_GAS_PROFILE_DIR=/tmp/sails-floor-vft cargo test -p benchmarks --features gas-profile --release minimal_vft_floor_bench -- --nocapture
```

Escalation checks:

```bash
rtk env SAILS_FLOOR_SAMPLES=5 SAILS_GAS_PROFILE_DIR=target/gstd-sails-lazy-pages-overhead/<candidate>/noop cargo test -p benchmarks --features gas-profile --release noop_floor_bench -- --nocapture
rtk env SAILS_FLOOR_SAMPLES=5 SAILS_GAS_PROFILE_DIR=target/gstd-sails-lazy-pages-overhead/<candidate>/vft cargo test -p benchmarks --features gas-profile --release minimal_vft_floor_bench -- --nocapture
```

Hygiene:

```bash
rtk cargo fmt -- --check
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
- Keep the scorecard current and compare against the refreshed baseline, not only old notes.
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
- Whether the winning reduction is in Sails, gstd, Gear lazy-pages, precharge/code shape, or no-winner.
- A short table of before/after gas for `noop_gstd`, `noop_sails`, `minimal_vft_hot_*`, and `minimal_vft_sails_*`.
- Wasm total/code/data deltas for changed programs.
- Lazy-pages bucket deltas and residual Wasm deltas where available.
- Paths to saved benchmark/profile artifacts.
- Files changed.
- Verification commands run and their results.
- Next recommended productionization step.
</output_contract>
