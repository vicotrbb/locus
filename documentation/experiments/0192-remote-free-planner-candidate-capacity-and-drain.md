# Experiment 0192: Remote-Free Planner Candidate Capacity And Drain

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0184-remote-free-planner-candidate-capacity-and-drain.md`

The postulate said that the
`increase_queue_capacity_and_drain_earlier` service candidate selected by
`RemoteFreeServiceRetuneCandidate` should restore the fixed queued-byte service
window when one owner shows both queue backpressure and retained-window drift.

## Change

Extended `remote_free_service_telemetry` with owner overrides that can change
queue capacity and queued-byte policy use for a single owner loop. The benchmark
now records `increase_capacity_and_drain_reports` in the sample and summary
output.

Added two static service cases:

- `one_capacity128_end_drain_owner`, where one owner uses queue capacity 128
  and end-drain behavior.
- `planner_candidate_capacity_and_drain_earlier`, where that owner uses the
  explicit combined candidate shape with the default 256-capacity queue and
  queued-byte policy drains.

The workload stayed fixed at four owner loops, 256 real `Vec<u8>` blocks per
owner, 4096 bytes per block, eight bursts of 32 blocks per owner, batch limit
64, and a 64-block, 262,144-byte queued-byte target.

## Commands

```text
cargo fmt --all
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The workspace test run reported 235 unit tests,
1 integration test, and 3 `locus_alloc` doctests passing.

The benchmark passed all five service telemetry cases. Key output:

```text
remote_free_service_telemetry_sample=one_capacity128_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 default_capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=12 drain_rounds=16 max_wait_bursts=4 mean_wait_bursts=1.875 observed_reports=32 reports_needing_retune=6 max_pending_over_target=64 max_queued_bytes_over_budget=262144 queue_backpressure_reports=4 keep_config_reports=26 drain_earlier_reports=2 increase_capacity_and_drain_reports=4 retune_candidate=increase_queue_capacity_and_drain_earlier
remote_free_service_telemetry_sample_summary=one_capacity128_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 default_capacity=256 batch_limit=64 retune_candidate=increase_queue_capacity_and_drain_earlier samples=8 reports_needing_retune_min=6 reports_needing_retune_max=6 reports_needing_retune_mean=6.000 max_pending_over_target_min=64 max_pending_over_target_max=64 max_pending_over_target_mean=64.000 max_queued_bytes_over_budget_min=262144 max_queued_bytes_over_budget_max=262144 max_queued_bytes_over_budget_mean=262144 keep_config_reports_min=26 keep_config_reports_max=26 keep_config_reports_mean=26.000 drain_earlier_reports_min=2 drain_earlier_reports_max=2 drain_earlier_reports_mean=2.000 increase_capacity_and_drain_reports_min=4 increase_capacity_and_drain_reports_max=4 increase_capacity_and_drain_reports_mean=4.000 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=1.875 mean_wait_max=1.875 mean_wait_mean=1.875
remote_free_service_telemetry_sample=planner_candidate_capacity_and_drain_earlier owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 default_capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=16 drain_rounds=16 max_wait_bursts=2 mean_wait_bursts=1.500 observed_reports=32 reports_needing_retune=0 max_pending_over_target=0 max_queued_bytes_over_budget=0 queue_backpressure_reports=0 keep_config_reports=32 drain_earlier_reports=0 increase_capacity_and_drain_reports=0 retune_candidate=keep_config
remote_free_service_telemetry_sample_summary=planner_candidate_capacity_and_drain_earlier owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 default_capacity=256 batch_limit=64 retune_candidate=keep_config samples=8 reports_needing_retune_min=0 reports_needing_retune_max=0 reports_needing_retune_mean=0.000 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 keep_config_reports_min=32 keep_config_reports_max=32 keep_config_reports_mean=32.000 drain_earlier_reports_min=0 drain_earlier_reports_max=0 drain_earlier_reports_mean=0.000 increase_capacity_and_drain_reports_min=0 increase_capacity_and_drain_reports_max=0 increase_capacity_and_drain_reports_mean=0.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run timing ranges:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `fixed_policy_all_clean` | 76.031 to 76.176 us | No change in performance detected |
| `one_end_drain_owner` | 77.733 to 77.887 us | No change in performance detected |
| `planner_candidate_drain_earlier` | 75.286 to 75.425 us | Performance has improved; 1 high mild outlier |
| `one_capacity128_end_drain_owner` | 79.281 to 79.452 us | 1 high severe outlier |
| `planner_candidate_capacity_and_drain_earlier` | 77.611 to 77.928 us | No note |

## Interpretation

The postulate survived this service benchmark.

The stressed capacity-128 end-drain owner selected
`increase_queue_capacity_and_drain_earlier`, produced four combined-action
reports, four queue-backpressure reports, six reports needing retune, max
pending over target 64, max queued bytes over budget 262,144, max wait 4
bursts, and mean wait 1.875 bursts.

The explicit combined candidate restored the fixed policy window: zero reports
needing retune, zero pending drift, zero queued-byte drift, zero queue
backpressure reports, 32 `keep_config` reports, max wait 2 bursts, and mean
wait 1.500 bursts.

This is not a production adaptive policy yet. It validates that both
service-planner candidates can be replayed as static benchmark cases before
runtime mutation is introduced.

## Next Question

Benchmark a dry-run adaptive planner that records which static candidate it
would apply over consecutive service windows without changing live policy.
