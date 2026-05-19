# CONTROL

## Status Contract

status_file: `PLAN.md`
attempt_log: `ATTEMPTS.md`
durable_notes: `NOTES.md`
results_file: `RESULTS.md`
update_memory_after: every_experiment
check_control_before: phase_change, strategic_pivot, expensive_step, public_api_change

## Human Priorities

primary_priority: low_gas
secondary_priority: evidence_quality

## Scope Knobs

allowed_files:
- `benchmarks/**`
- `rs/storage/**`
- `docs/goals/vft-storage-better-solution/**`
- `docs/goals/vft-storage-optimization/**`

protected_files:
- unrelated examples and client code unless benchmark wiring requires them
- Sails dispatcher/routing internals

max_blast_radius: storage primitives, VFT/storage benchmarks, and goal documentation

## Acceptance Thresholds

preferred_success: 7_percent_weighted_improvement_vs_generic_sails_static
strong_narrow_success: 10_percent_on_two_hot_ops_with_max_3_percent_regression
exploratory_progress: 3_percent_weighted_improvement_with_profile_evidence
no_winner_is_completion: false
large_capacity_target: 1_000_000_entries

## Resource Knobs

max_runtime_per_fast_step: 15_minutes
max_runtime_per_expensive_step: none
max_parallel_jobs: use_judgment
network_allowed: false

## Decision Gates

require_approval_for:
- no_winner_stop
- strategic_pivot
- dependency_change
- public_api_change
- scope_expansion
- destructive_change

## Latest Human Nudge

The goal should be to find a better solution: optimize existing, tweak the best performer, or develop something better.
