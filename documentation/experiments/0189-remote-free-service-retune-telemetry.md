# Experiment 0189: Remote-Free Service Retune Telemetry

Date: 2026-07-03

## Postulate

[Postulate 0181](../postulates/0181-remote-free-service-retune-telemetry.md)
claimed that Locus should expose a service-level telemetry summary that
aggregates queued-byte drift reports from multiple owner loops before mutating
remote-free policy adaptively.

## Change

Added `RemoteFreeServiceRetuneSummary` and
`RemoteFreeRetuneActionCounts`.

The summary observes `RemoteFreeQueuedByteDriftReport` values and records:

- observed report count;
- reports needing retune;
- maximum pending items over target;
- maximum queued bytes over budget;
- queue backpressure report count;
- counts for each `RemoteFreeQueuedByteRetuneAction`.

The summary is diagnostic only. It does not mutate queue capacity, owner drain
cadence, queued-byte budgets, or owner-loop release behavior.

Added `remote_free_service_telemetry`, a Criterion benchmark with four real
owner loops. Each owner allocates and remotely frees 256 `Vec<u8>` blocks of
4096 bytes through `RemoteFreeQueue` and `RemoteFreeDrainController`.

The benchmark compares:

- `fixed_policy_all_clean`: all four owners use the fixed queued-byte policy;
- `one_end_drain_owner`: one owner defers until end-drain while the remaining
  three owners use the fixed queued-byte policy.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free::telemetry
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Focused telemetry tests passed: 3 passed, 0 failed.
- Service telemetry benchmark passed and asserted every expected counter.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 229 unit tests, 1 integration test, and 3
  `locus_alloc` doctests passed.

## Service Telemetry Results

Single-sample service telemetry:

```text
remote_free_service_telemetry_sample=fixed_policy_all_clean owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=16 drain_rounds=16 max_wait_bursts=2 mean_wait_bursts=1.500 observed_reports=32 reports_needing_retune=0 max_pending_over_target=0 max_queued_bytes_over_budget=0 queue_backpressure_reports=0 keep_config_reports=32 drain_earlier_reports=0
remote_free_service_telemetry_sample=one_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=1024 drained_count=1024 released_bytes=4194304 policy_drains=12 drain_rounds=16 max_wait_bursts=8 mean_wait_bursts=2.250 observed_reports=32 reports_needing_retune=6 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 keep_config_reports=26 drain_earlier_reports=6
```

Repeated service telemetry summaries:

```text
remote_free_service_telemetry_sample_summary=fixed_policy_all_clean owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 reports_needing_retune_min=0 reports_needing_retune_max=0 reports_needing_retune_mean=0.000 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 keep_config_reports_min=32 keep_config_reports_max=32 keep_config_reports_mean=32.000 drain_earlier_reports_min=0 drain_earlier_reports_max=0 drain_earlier_reports_mean=0.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_service_telemetry_sample_summary=one_end_drain_owner owners=4 blocks_per_owner=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 reports_needing_retune_min=6 reports_needing_retune_max=6 reports_needing_retune_mean=6.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 keep_config_reports_min=26 keep_config_reports_max=26 keep_config_reports_mean=26.000 drain_earlier_reports_min=6 drain_earlier_reports_max=6 drain_earlier_reports_mean=6.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=2.250 mean_wait_max=2.250 mean_wait_mean=2.250
```

Short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_fixed_policy_all_clean` | 75.540 us to 75.816 us | No regression note emitted |
| `remote_free_service_telemetry_one_end_drain_owner` | 77.595 us to 77.753 us | 1 high severe outlier |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

The clean fixed-policy service produced 32 reports and all 32 reported
`keep_config`. It retained the per-owner fixed queued-byte behavior: 16 total
policy drains, max wait 2 bursts, mean wait 1.500 bursts, and no drift.

The mixed service isolated one drifting owner without changing release logic:
six of 32 reports needed retune, all six recommended `drain_earlier`, and the
service summary preserved the worst retained-window drift at 192 pending items
over target and 786,432 bytes over budget. The other 26 reports remained
`keep_config`.

This gives adaptive policy work a measured service-level observation source,
but it still does not prove any adaptive mutation is safe. The next step must
benchmark a concrete policy change against this fixed-policy baseline.

## Next Step

Define a non-mutating adaptive candidate planner that consumes
`RemoteFreeServiceRetuneSummary` and proposes the next benchmark case without
changing runtime policy.
