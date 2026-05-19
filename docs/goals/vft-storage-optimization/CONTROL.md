# CONTROL

## Status Contract

status_file: `PLAN.md`
attempt_log: `ATTEMPTS.md`
durable_notes: `NOTES.md`
results_file: `RESULTS.md`
update_memory_after: every_experiment
check_control_before: phase_change, strategic_pivot, expensive_step, public_api_change

## Human Priorities

primary_priority: evidence_quality
secondary_priority: low_gas

## Scope Knobs

allowed_files:
- `benchmarks/**`
- `rs/storage/**`
- `docs/goals/vft-storage-optimization/**`

protected_files:
- unrelated examples and client code unless benchmark wiring requires them
- Sails dispatcher/routing internals

max_blast_radius: storage primitives, VFT/storage benchmarks, and goal documentation

## Acceptance Thresholds

beat_hashmap_btree_by: 25_percent_weighted_hot_path
replace_sails_static_if: 10_percent_on_two_hot_ops_or_7_percent_weighted_total
max_allowed_hot_op_regression_vs_sails_static: 3_percent
minimum_candidates_or_rejections: 3
large_capacity_target: 1_000_000_entries

## Resource Knobs

max_runtime_per_fast_step: 15_minutes
max_runtime_per_expensive_step: none
max_parallel_jobs: use_judgment
network_allowed: false

## Decision Gates

require_approval_for:
- strategic_pivot
- dependency_change
- public_api_change
- scope_expansion
- destructive_change

## Sidecar Inputs

sidecar_apply_cadence: between_runs_only
nudge_file: none
human_overlay_file: none
review_queue_file: none

## Latest Human Nudge

Approved the goal shape and thresholds on 2026-05-19.
