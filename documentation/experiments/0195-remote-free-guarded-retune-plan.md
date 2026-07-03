# Experiment 0195: Remote-Free Guarded Retune Plan

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0187-remote-free-guarded-retune-plan.md`

The postulate said that a guarded service retune planner should turn dry-run
stability into explicit plan decisions: apply an actionable candidate only
after the configured stable-window count, confirm it only after a clean
follow-up service window, roll it back if the follow-up window still needs
retuning, and stop applying candidates after the configured mutation limit.

## Change

Added:

- `RemoteFreeServiceRetuneGuard`;
- `RemoteFreeServiceRetuneGuardDecision`;
- `RemoteFreeServiceRetuneGuardError`;
- `RemoteFreeServiceRetuneDryRunPlanner::reset`.

The guard wraps the existing dry-run planner. It emits explicit decisions but
does not mutate policy by itself. Callers still choose how to apply a candidate
and must feed the next service summary back to the guard for confirmation or
rollback.

Extended `remote_free_service_telemetry` with two guarded sequences:

- `remote_free_service_telemetry_guarded_confirming`, which applies
  `drain_earlier`, confirms it with a clean candidate window, then applies and
  confirms `increase_queue_capacity_and_drain_earlier`;
- `remote_free_service_telemetry_guarded_rollback`, which applies
  `drain_earlier` and then validates it against another drifting end-drain
  window to force rollback.

Both sequences use real owner-loop cases with real `Vec<u8>` allocations and
remote-free release accounting.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc remote_free:: -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused remote-free test run reported 71
remote-free tests passing. The workspace test run reported 245 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing.

Guarded confirming sequence:

```text
remote_free_service_guarded_confirming_sample windows=7 stable_windows=2 max_mutations=2 submitted_count=7168 drained_count=7168 released_bytes=29360128 policy_drains=96 observed_reports=224 reports_needing_retune=24 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=8 hold_decisions=3 apply_decisions=2 confirmed_decisions=2 rollback_decisions=0 mutation_limit_decisions=0 drain_earlier_apply_decisions=1 combined_apply_decisions=1 max_wait_bursts=8 mean_wait_bursts=1.821 final_pending_candidate=none final_applied_mutations=2 final_confirmed_mutations=2 final_rollbacks=0
remote_free_service_guarded_confirming_sample_summary windows=7 stable_windows=2 max_mutations=2 samples=8 reports_needing_retune_min=24 reports_needing_retune_max=24 reports_needing_retune_mean=24.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 queue_backpressure_reports_min=8 queue_backpressure_reports_max=8 queue_backpressure_reports_mean=8.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=2 confirmed_decisions_max=2 confirmed_decisions_mean=2.000 rollback_decisions_min=0 rollback_decisions_max=0 rollback_decisions_mean=0.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=1.821 mean_wait_max=1.821 mean_wait_mean=1.821 final_pending_candidate=none final_applied_mutations=2 final_confirmed_mutations=2 final_rollbacks=0
```

Guarded rollback sequence:

```text
remote_free_service_guarded_rollback_sample windows=4 stable_windows=2 max_mutations=2 submitted_count=4096 drained_count=4096 released_bytes=16777216 policy_drains=52 observed_reports=128 reports_needing_retune=18 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 hold_decisions=2 apply_decisions=1 confirmed_decisions=0 rollback_decisions=1 mutation_limit_decisions=0 drain_earlier_apply_decisions=1 combined_apply_decisions=0 max_wait_bursts=8 mean_wait_bursts=2.062 final_pending_candidate=none final_applied_mutations=1 final_confirmed_mutations=0 final_rollbacks=1
remote_free_service_guarded_rollback_sample_summary windows=4 stable_windows=2 max_mutations=2 samples=8 reports_needing_retune_min=18 reports_needing_retune_max=18 reports_needing_retune_mean=18.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 queue_backpressure_reports_min=0 queue_backpressure_reports_max=0 queue_backpressure_reports_mean=0.000 apply_decisions_min=1 apply_decisions_max=1 apply_decisions_mean=1.000 confirmed_decisions_min=0 confirmed_decisions_max=0 confirmed_decisions_mean=0.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=2.062 mean_wait_max=2.062 mean_wait_mean=2.062 final_pending_candidate=none final_applied_mutations=1 final_confirmed_mutations=0 final_rollbacks=1
```

Short-run timing ranges:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_guarded_confirming` | 582.68 to 588.08 us | No change in performance detected; 2 high mild outliers |
| `remote_free_service_telemetry_guarded_rollback` | 332.75 to 339.32 us | No change in performance detected |

## Interpretation

The postulate survived this benchmark.

The guard did not emit apply decisions before the two-window stability
threshold. In the confirming sequence it emitted two apply decisions, confirmed
both with clean candidate windows, left no pending candidate, and preserved
7168 submitted blocks, 7168 drained blocks, and 29,360,128 released bytes.

In the rollback sequence it emitted one apply decision for `drain_earlier`,
observed a non-clean validation window, returned one rollback decision, left no
pending candidate, and preserved 4096 submitted blocks, 4096 drained blocks,
and 16,777,216 released bytes.

This is still an explicit planning layer. The guard does not mutate production
policy directly; callers must apply candidates and feed validation telemetry
back into the guard.

## Next Question

Measure the mutation-limit path with real service windows, then define the
smallest production-facing policy application API.
