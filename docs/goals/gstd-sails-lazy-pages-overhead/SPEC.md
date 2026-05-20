# SPEC: Reduce Gear/gstd/Sails Lazy-Pages Overhead

## Goal

Investigate and reduce fixed Gear execution overhead that appears before storage-map tuning can dominate. The target surface is the Gear/gstd/Sails stack used by VFT-style programs: raw WAT lower bound, minimal `gstd`, generated Sails noop, minimal production-shaped Sails VFT, manual Sails-wire hot path, and Gear lazy-pages buckets.

## Current Baseline

Use `docs/gear-sails-noop-overhead-2026-05-20.md` as the starting snapshot:

- `noop_gstd`: 444,865,660 gas
- `noop_sails`: 587,088,644 gas
- `vft_framework_noop`: 862,096,376 gas
- `minimal_vft_hot_transfer`: 850,864,350 gas
- `minimal_vft_sails_transfer`: 1,115,816,387 gas
- Generated Sails minimal VFT is about 37.5 KiB; manual hot path is about 15.9 KiB.

## Scope

In scope:

- Re-run and improve the floor benchmark matrix.
- Attribute gas to precharge, lazy-pages, residual Wasm, code/data size, route/reply shape, and Gear host/runtime buckets where available.
- Implement and measure targeted reductions in Sails/gstd program shape or Gear lazy-pages overhead.
- Prototype generated or macro-supported Sails hot dispatch if it can preserve the Sails wire contract.
- Save benchmark/profile artifacts and explain the decision.

Out of scope:

- Further storage-map architecture work unless the overhead investigation proves a storage-specific interaction.
- Broad Sails API redesign.
- Publishing experimental APIs without benchmark proof.
- Hand-waving based on native microbenchmarks only.

## Candidate Tracks

1. Sails hot dispatch: turn the benchmark-only manual Sails-wire hot path into a generated or opt-in Sails path for simple Gear-native services.
2. Sails/gstd size reduction: remove framework/code/data pulled into noops or minimal VFT paths when not needed.
3. Lazy-pages attribution and reduction: prove why noop spends around 140-150M in lazy-pages, then test Gear-side or program-shape changes that reduce touched pages or init/post-processing cost.
4. Precharge reduction: identify whether `instrumented_code`, `module_instantiation`, metadata, or data sections drive the VFT framework floor and reduce the binary shape.

## Scorecard

Primary score: median release-mode gas in `noop_floor_bench` and `minimal_vft_floor_bench`.

Passing thresholds:

- Reduce `noop_sails - noop_gstd` by at least 25%, or prove with source-level evidence why the current Sails noop floor cannot be reduced in this repo.
- Reduce `minimal_vft_sails_* - minimal_vft_hot_*` by at least 25% for two hot VFT operations, or implement enough generated hot-path infrastructure that a follow-up can close the remaining gap.
- No optimized row may regress `noop_gstd` or raw WAT by more than 3%.
- Wasm size changes must be reported with code/data section deltas.

Stop condition: one validated overhead reduction lands, or at least three credible reduction candidates are rejected with measurement and code-level rationale.

## Done When

- `GOAL.md`, `PLAN.md`, `ATTEMPTS.md`, `NOTES.md`, `CONTROL.md`, and `RESULTS.md` exist for this goal.
- Baselines are refreshed and saved under a repo or target artifact path named for this goal.
- At least three candidates are attempted or rejected with evidence in `ATTEMPTS.md`.
- `RESULTS.md` contains before/after gas, section-size deltas, lazy-pages bucket deltas, and the final recommendation.
- Focused benchmarks and hygiene checks pass or their failures are documented with impact.

## Open Approval Point

Before running this as a long Codex `/goal`, approve or adjust the numeric thresholds above, especially the 25% reduction targets and the 3% lower-bound regression cap.
