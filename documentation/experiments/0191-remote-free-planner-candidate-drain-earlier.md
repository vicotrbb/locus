# Experiment 0191: Remote-Free Planner Candidate Drain Earlier

Date: 2026-07-03

## Postulate

[Postulate 0183](../postulates/0183-remote-free-planner-candidate-drain-earlier.md)
claimed that the `drain_earlier` service candidate selected by
`RemoteFreeServiceRetuneCandidate` should restore the fixed queued-byte
service window when benchmarked as an explicit candidate case, without adding
adaptive runtime mutation.

## Change

Extended `remote_free_service_telemetry` with
`planner_candidate_drain_earlier`.

The new case reuses the same four owner loops and applies the queued-byte
policy to the owner that drifted in the `one_end_drain_owner` baseline. This
keeps the candidate explicit as a benchmark case. It does not let telemetry
mutate live policy.

The workload remains unchanged:

- four owner loops;
- 256 `Vec<u8>` blocks per owner;
- 4096 bytes per block;
- eight bursts of 32 blocks per owner;
- queue capacity 256;
- drain batch limit 64;
- 64-block, 262,144-byte queued-byte target.

## Validation

Commands:

```bash
cargo fmt --all
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Service telemetry benchmark passed and asserted all three cases.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 235 unit tests, 1 integration test, and 3
  `locus_alloc` doctests passed.

## Candidate Results

Single-sample service telemetry:

```text
remote_free_service_telemetry_sample=fixed_policy_all_clean owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=16 drain_rounds=16 max_wait_bursts=2 mean_wait_bursts=1.500 observed_reports=32 reports_needing_retune=0 max_pending_over_target=0 max_queued_bytes_over_budget=0 queue_backpressure_reports=0 keep_config_reports=32 drain_earlier_reports=0 retune_candidate=keep_config
remote_free_service_telemetry_sample=one_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=12 drain_rounds=16 max_wait_bursts=8 mean_wait_bursts=2.250 observed_reports=32 reports_needing_retune=6 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 keep_config_reports=26 drain_earlier_reports=6 retune_candidate=drain_earlier
remote_free_service_telemetry_sample=planner_candidate_drain_earlier owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=16 drain_rounds=16 max_wait_bursts=2 mean_wait_bursts=1.500 observed_reports=32 reports_needing_retune=0 max_pending_over_target=0 max_queued_bytes_over_budget=0 queue_backpressure_reports=0 keep_config_reports=32 drain_earlier_reports=0 retune_candidate=keep_config
```

Repeated service telemetry summaries:

```text
remote_free_service_telemetry_sample_summary=fixed_policy_all_clean owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_candidate=keep_config samples=8 reports_needing_retune_min=0 reports_needing_retune_max=0 reports_needing_retune_mean=0.000 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 keep_config_reports_min=32 keep_config_reports_max=32 keep_config_reports_mean=32.000 drain_earlier_reports_min=0 drain_earlier_reports_max=0 drain_earlier_reports_mean=0.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_service_telemetry_sample_summary=one_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_candidate=drain_earlier samples=8 reports_needing_retune_min=6 reports_needing_retune_max=6 reports_needing_retune_mean=6.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 keep_config_reports_min=26 keep_config_reports_max=26 keep_config_reports_mean=26.000 drain_earlier_reports_min=6 drain_earlier_reports_max=6 drain_earlier_reports_mean=6.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=2.250 mean_wait_max=2.250 mean_wait_mean=2.250
remote_free_service_telemetry_sample_summary=planner_candidate_drain_earlier owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_candidate=keep_config samples=8 reports_needing_retune_min=0 reports_needing_retune_max=0 reports_needing_retune_mean=0.000 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 keep_config_reports_min=32 keep_config_reports_max=32 keep_config_reports_mean=32.000 drain_earlier_reports_min=0 drain_earlier_reports_max=0 drain_earlier_reports_mean=0.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_fixed_policy_all_clean` | 76.102 us to 76.229 us | No change in performance detected |
| `remote_free_service_telemetry_one_end_drain_owner` | 77.744 us to 77.791 us | Change within noise threshold, 1 low mild outlier |
| `remote_free_service_telemetry_planner_candidate_drain_earlier` | 76.696 us to 77.862 us | 1 high severe outlier |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

The one-owner end-drain baseline selected `drain_earlier` and showed retained
window drift: six reports needing retune, 192 pending items over target,
786,432 queued bytes over budget, max wait 8 bursts, and mean wait 2.250
bursts.

The explicit planner candidate restored the fixed queued-byte service window:
zero reports needing retune, zero retained-window drift, 32 `keep_config`
reports, max wait 2 bursts, and mean wait 1.500 bursts. It did this as a
static benchmark case, not as runtime mutation driven by telemetry.

## Next Step

Extend the explicit candidate benchmark to a mixed service where one owner has
queue backpressure and retained-window drift, so the
`increase_queue_capacity_and_drain_earlier` candidate is measured before any
adaptive policy is introduced.
