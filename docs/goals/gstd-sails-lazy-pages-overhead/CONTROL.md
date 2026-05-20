# CONTROL

## Status Contract

status_file: `PLAN.md`
attempt_log: `ATTEMPTS.md`
durable_notes: `NOTES.md`
results_file: `RESULTS.md`
check_control_before: phase_change, strategic_pivot, expensive_step, gear_side_change, public_api_change

## Human Priorities

primary_priority: lower_fixed_gas_floor
secondary_priority: productionizable_sails_path
tertiary_priority: explain_lazy_pages_cost

## Scope Knobs

allowed_files:
- `benchmarks/**`
- `rs/**`
- `docs/goals/gstd-sails-lazy-pages-overhead/**`
- `docs/gear-sails-noop-overhead-2026-05-20.md`

conditional_files:
- `../gear/protocol/lazy-pages/**`
- `../gear/ethexe/processor/src/host/api/lazy_pages.rs`

protected_files:
- unrelated examples and client code unless benchmark wiring requires them
- storage-map architecture files unless an overhead candidate directly needs them

max_blast_radius: Sails/gstd overhead benchmarks, narrowly scoped Sails hot-path generation, and Gear lazy-pages investigation

## Acceptance Thresholds

reduce_noop_sails_delta_vs_gstd_by: 25_percent
reduce_minimal_vft_sails_delta_vs_hot_by: 25_percent_on_two_hot_ops
max_lower_bound_regression: 3_percent
minimum_candidates_or_rejections: 3

## Resource Knobs

baseline_artifact_root: `benchmarks/target/gstd-sails-lazy-pages-overhead`
fast_samples: 1
final_samples: 5
max_runtime_per_fast_step: 10_minutes
max_runtime_per_escalation_step: 60_minutes
network_allowed: false

## Decision Gates

require_approval_for:
- gear_side_change
- public_api_change
- dependency_change
- scope_expansion
- weakening_thresholds
- destructive_change

## Latest Human Nudge

Approved numeric thresholds on 2026-05-20.
