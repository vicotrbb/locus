# Experiment 0193: Remote-Free Dry-Run Service Planner

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0185-remote-free-dry-run-service-planner.md`

The postulate said that a dry-run planner over
`RemoteFreeServiceRetuneSummary` should record candidate changes across
consecutive service windows and expose a stable would-apply candidate without
mutating queue capacity, drain policy, or queued-byte budgets.

## Change

Added `RemoteFreeServiceRetuneDryRunPlanner` and
`RemoteFreeServiceRetuneDryRunPlannerError`.

The planner:

- observes service summaries as discrete windows;
- tracks the latest service candidate;
- tracks the consecutive actionable-candidate streak;
- exposes `would_apply_candidate` only after the configured non-zero stability
  window;
- resets the streak on `keep_config`, `collect_telemetry`,
  `review_queued_byte_budget`, and candidate changes;
- never mutates remote-free queue capacity, drain cadence, or byte budgets.

Extended `remote_free_service_telemetry` with a dry-run sequence benchmark that
uses the existing real owner-loop service cases:

1. clean fixed-policy window;
2. end-drain window selecting `drain_earlier`;
3. repeated end-drain window reaching dry-run stability;
4. capacity-128 end-drain window selecting the combined candidate;
5. repeated capacity-128 end-drain window reaching dry-run stability;
6. clean fixed-policy window resetting the planner.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc remote_free::planner -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused planner test run reported 11
planner tests passing. The workspace test run reported 240 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing.

The dry-run sequence benchmark passed with the real allocation and release
path intact:

```text
remote_free_service_dry_run_sample windows=6 stable_windows=2 submitted_count=6144 drained_count=6144 released_bytes=25165824 policy_drains=80 observed_reports=192 reports_needing_retune=24 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=8 keep_config_candidate_windows=2 drain_earlier_candidate_windows=2 combined_candidate_windows=2 would_apply_drain_earlier_windows=1 would_apply_combined_windows=1 max_wait_bursts=8 mean_wait_bursts=1.875 final_candidate=keep_config final_streak=0 final_would_apply=none
remote_free_service_dry_run_sample_summary windows=6 stable_windows=2 samples=8 reports_needing_retune_min=24 reports_needing_retune_max=24 reports_needing_retune_mean=24.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 queue_backpressure_reports_min=8 queue_backpressure_reports_max=8 queue_backpressure_reports_mean=8.000 would_apply_drain_earlier_windows_min=1 would_apply_drain_earlier_windows_max=1 would_apply_drain_earlier_windows_mean=1.000 would_apply_combined_windows_min=1 would_apply_combined_windows_max=1 would_apply_combined_windows_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=1.875 mean_wait_max=1.875 mean_wait_mean=1.875 final_candidate=keep_config final_streak=0 final_would_apply=none
```

Short-run timing range:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_dry_run_sequence` | 490.32 to 491.70 us | No note |

The full benchmark run also kept the prior static service cases passing. Their
short-run timings shifted around previous Criterion baselines, so treat those
regression and improvement labels as local run noise unless a longer benchmark
run confirms them.

## Interpretation

The postulate survived this benchmark.

The dry-run planner observed six real service windows and correctly produced
exactly one would-apply `drain_earlier` window and one would-apply
`increase_queue_capacity_and_drain_earlier` window after two consecutive
matching candidate windows. It then reset to `keep_config` with final streak 0
and `final_would_apply=none` when clean telemetry returned.

The sequence preserved real allocation and release accounting: 6144 submitted
blocks, 6144 drained blocks, 25,165,824 released bytes, 80 policy drains, max
wait 8 bursts, and mean wait 1.875 bursts.

This is still not a live adaptive policy. It validates that service telemetry
can produce a stable non-mutating plan signal across windows.

## Next Question

Define the first live adaptive policy candidate behind explicit guardrails, or
add a longer mixed-window benchmark that tests dry-run stability against
oscillating candidates before mutation.
