# Experiment 0194: Remote-Free Dry-Run Oscillation

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0186-remote-free-dry-run-oscillation.md`

The postulate said that the dry-run service planner should reject oscillating
actionable candidates. With a two-window stability requirement, alternating
`drain_earlier` and `increase_queue_capacity_and_drain_earlier` service
windows should never expose a would-apply candidate.

## Change

Extended the dry-run service benchmark with
`remote_free_service_telemetry_dry_run_oscillation`.

The dry-run harness now models two service-window sequences:

- a stable sequence from Experiment 0193 that repeats each actionable
  candidate and reaches a would-apply signal;
- an oscillating sequence that alternates `drain_earlier` and
  `increase_queue_capacity_and_drain_earlier` before returning to clean
  telemetry.

Both sequences reuse the same real owner-loop service cases and the same
two-window stability requirement.

## Commands

```text
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The workspace test run reported 240 unit tests,
1 integration test, and 3 `locus_alloc` doctests passing.

The oscillating dry-run sequence passed with zero would-apply windows:

```text
remote_free_service_dry_run_oscillation_sample windows=6 stable_windows=2 submitted_count=6144 drained_count=6144 released_bytes=25165824 policy_drains=80 observed_reports=192 reports_needing_retune=24 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=8 keep_config_candidate_windows=2 drain_earlier_candidate_windows=2 combined_candidate_windows=2 would_apply_drain_earlier_windows=0 would_apply_combined_windows=0 max_wait_bursts=8 mean_wait_bursts=1.875 final_candidate=keep_config final_streak=0 final_would_apply=none
remote_free_service_dry_run_oscillation_sample_summary windows=6 stable_windows=2 samples=8 reports_needing_retune_min=24 reports_needing_retune_max=24 reports_needing_retune_mean=24.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 queue_backpressure_reports_min=8 queue_backpressure_reports_max=8 queue_backpressure_reports_mean=8.000 would_apply_drain_earlier_windows_min=0 would_apply_drain_earlier_windows_max=0 would_apply_drain_earlier_windows_mean=0.000 would_apply_combined_windows_min=0 would_apply_combined_windows_max=0 would_apply_combined_windows_mean=0.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=1.875 mean_wait_max=1.875 mean_wait_mean=1.875 final_candidate=keep_config final_streak=0 final_would_apply=none
```

Short-run timing range:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_dry_run_oscillation` | 486.99 to 488.59 us | 1 high mild outlier |

The stable dry-run sequence still produced one would-apply `drain_earlier`
window and one would-apply combined-candidate window in the same run.

## Interpretation

The postulate survived this benchmark.

The oscillating sequence used the same six real service windows, 6144
submitted blocks, 6144 drained blocks, 25,165,824 released bytes, 80 policy
drains, max wait 8 bursts, and mean wait 1.875 bursts as the stable sequence
composition. Because the actionable candidates alternated, the dry-run planner
kept both `would_apply_drain_earlier_windows` and
`would_apply_combined_windows` at zero.

This strengthens the guardrail before live mutation: unstable candidate
telemetry does not become an adaptive policy signal.

## Next Question

Define the first guarded live adaptive policy benchmark with explicit mutation
limits, rollback conditions, and allocation-counter checks.
