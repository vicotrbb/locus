# Experiment 0180: Remote-Free Drift Retune Hint

Date: 2026-07-03

## Postulate

[Postulate 0172](../postulates/0172-remote-free-drift-retune-hint.md)
claimed that `RemoteFreeQueuedByteDriftReport` should expose a typed retune
hint that classifies the first diagnostic response to observed drift without
mutating the remote-free drain policy.

## Change

Added `RemoteFreeQueuedByteRetuneHint` with these variants:

- `KeepConfig`;
- `IncreaseQueueCapacity`;
- `ReviewDrainCadence`;
- `ReviewQueuedByteBudget`;
- `ReviewMultipleSignals`.

Added:

- `RemoteFreeQueuedByteDriftReport::retune_hint()`;
- `RemoteFreeQueuedByteRetuneHint::as_str()`;
- public re-exports through `locus_alloc`;
- focused tests for hint classification and stable labels;
- `retune_hint=...` output in `remote_free_drift_matrix`;
- drift diagnostics documentation in the queued-byte budget selection note.

The hint is diagnostic only. It does not mutate policy or queue sizing.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc queued_byte
cargo bench -p locus-alloc --bench remote_free_drift_matrix -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Focused queued-byte tests passed: 34 passed, 0 failed.
- Drift matrix benchmark passed and asserted the expected hint for every case.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 224 unit tests and 3 `locus_alloc` doctests passed.

## Drift Matrix Hint Results

Final short-run sample summaries:

```text
remote_free_drift_matrix_sample_summary=matched_end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=256 target_pending=256 queued_byte_budget=2621440 retune_hint=keep_config samples=8 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000 full_min=0 full_max=0 full_mean=0.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440
remote_free_drift_matrix_sample_summary=pending_target64_budget_total blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=256 target_pending=64 queued_byte_budget=2621440 retune_hint=review_drain_cadence samples=8 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000 full_min=0 full_max=0 full_mean=0.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440
remote_free_drift_matrix_sample_summary=pending_target256_budget640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=256 target_pending=256 queued_byte_budget=655360 retune_hint=review_queued_byte_budget samples=8 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=1966080 max_queued_bytes_over_budget_max=1966080 max_queued_bytes_over_budget_mean=1966080 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000 full_min=0 full_max=0 full_mean=0.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440
remote_free_drift_matrix_sample_summary=capacity64_backpressure blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 target_pending=64 queued_byte_budget=655360 retune_hint=increase_queue_capacity samples=8 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=1 queue_backpressure_observed_max=1 queue_backpressure_observed_mean=1.000 full_min=3 full_max=3 full_mean=3.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360
```

Final short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_drift_matrix_matched_end_drain` | 42.958 us to 43.169 us | No change in performance detected |
| `remote_free_drift_matrix_pending_target64_budget_total` | 42.975 us to 43.086 us | No change in performance detected, 1 low mild outlier |
| `remote_free_drift_matrix_pending_target256_budget640kib` | 42.578 us to 42.703 us | Change within noise threshold, 2 low mild outliers |
| `remote_free_drift_matrix_capacity64_backpressure` | 38.212 us to 38.606 us | Performance has improved, 2 outliers |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

The retune hint correctly maps:

- no drift to `keep_config`;
- pending drift to `review_drain_cadence`;
- queued-byte drift to `review_queued_byte_budget`;
- queue backpressure to `increase_queue_capacity`;
- mixed unit-test drift to `review_multiple_signals`.

This keeps adaptive policy work staged behind an explicit diagnostic layer.

## Next Step

Benchmark one candidate action at a time from the hint outputs. The first
candidate should use the backpressure hint to test whether increasing queue
capacity removes `full_count` without hiding unacceptable release latency.
