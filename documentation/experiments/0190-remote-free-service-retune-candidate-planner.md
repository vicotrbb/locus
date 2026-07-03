# Experiment 0190: Remote-Free Service Retune Candidate Planner

Date: 2026-07-03

## Postulate

[Postulate 0182](../postulates/0182-remote-free-service-retune-candidate-planner.md)
claimed that `RemoteFreeServiceRetuneSummary` should feed a non-mutating
candidate planner that recommends the next remote-free benchmark case without
changing runtime queue capacity, drain cadence, or queued-byte budgets.

## Change

Added `RemoteFreeServiceRetuneCandidate` with stable labels:

- `collect_telemetry`;
- `keep_config`;
- `increase_queue_capacity`;
- `drain_earlier`;
- `review_queued_byte_budget`;
- `increase_queue_capacity_and_drain_earlier`.

Added `RemoteFreeServiceRetuneCandidate::from_summary` as a non-mutating
planner over `RemoteFreeServiceRetuneSummary`.

The planner uses a conservative priority order:

- empty telemetry reports `collect_telemetry`;
- clean service telemetry reports `keep_config`;
- combined capacity and retained-window drift reports
  `increase_queue_capacity_and_drain_earlier`;
- retained-window drift reports `drain_earlier`;
- byte-shape drift reports `review_queued_byte_budget`;
- queue backpressure alone reports `increase_queue_capacity`.

Updated `remote_free_service_telemetry` to print and assert the selected
candidate while keeping the real owner-loop allocation, drain, and telemetry
counters unchanged.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free::planner
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Focused planner tests passed: 6 passed, 0 failed.
- Service telemetry benchmark passed and asserted both selected candidates.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 235 unit tests, 1 integration test, and 3
  `locus_alloc` doctests passed.

## Candidate Results

Single-sample service telemetry:

```text
remote_free_service_telemetry_sample=fixed_policy_all_clean owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=16 drain_rounds=16 max_wait_bursts=2 mean_wait_bursts=1.500 observed_reports=32 reports_needing_retune=0 max_pending_over_target=0 max_queued_bytes_over_budget=0 queue_backpressure_reports=0 keep_config_reports=32 drain_earlier_reports=0 retune_candidate=keep_config
remote_free_service_telemetry_sample=one_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=12 drain_rounds=16 max_wait_bursts=8 mean_wait_bursts=2.250 observed_reports=32 reports_needing_retune=6 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 keep_config_reports=26 drain_earlier_reports=6 retune_candidate=drain_earlier
```

Repeated service telemetry summaries:

```text
remote_free_service_telemetry_sample_summary=fixed_policy_all_clean owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_candidate=keep_config samples=8 reports_needing_retune_min=0 reports_needing_retune_max=0 reports_needing_retune_mean=0.000 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 keep_config_reports_min=32 keep_config_reports_max=32 keep_config_reports_mean=32.000 drain_earlier_reports_min=0 drain_earlier_reports_max=0 drain_earlier_reports_mean=0.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_service_telemetry_sample_summary=one_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_candidate=drain_earlier samples=8 reports_needing_retune_min=6 reports_needing_retune_max=6 reports_needing_retune_mean=6.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 keep_config_reports_min=26 keep_config_reports_max=26 keep_config_reports_mean=26.000 drain_earlier_reports_min=6 drain_earlier_reports_max=6 drain_earlier_reports_mean=6.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=2.250 mean_wait_max=2.250 mean_wait_mean=2.250
```

Short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_fixed_policy_all_clean` | 75.730 us to 75.962 us | Change within noise threshold, 3 outliers |
| `remote_free_service_telemetry_one_end_drain_owner` | 77.865 us to 77.970 us | Change within noise threshold, 3 outliers |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

The planner distinguished empty telemetry, clean telemetry, queue
backpressure, retained-window drift, byte-shape drift, and combined
backpressure plus retained-window drift in focused tests.

In the real service benchmark, fixed queued-byte policy still selected
`keep_config` with 32 clean reports, max wait 2 bursts, and mean wait 1.500
bursts. The one-owner drift service selected `drain_earlier` with six
drifting reports, max wait 8 bursts, mean wait 2.250 bursts, and no policy
mutation.

The candidate planner is now a safe boundary between service telemetry and the
next adaptive benchmark. It selects what to benchmark next without changing
runtime policy.

## Next Step

Benchmark an explicit `drain_earlier` candidate chosen by the planner against
the one-owner end-drain service baseline.
