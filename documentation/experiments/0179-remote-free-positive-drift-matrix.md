# Experiment 0179: Remote-Free Positive Drift Matrix

Date: 2026-07-03

## Postulate

[Postulate 0171](../postulates/0171-remote-free-positive-drift-matrix.md)
claimed that `RemoteFreeQueuedByteDriftReport` should be validated against
deliberately mis-sized real allocation traces, not only zero-drift unit tests
and the known-good queued-byte config.

## Change

Added `remote_free_drift_matrix`, a focused Criterion benchmark target that
uses:

- real `Vec<u8>` allocation blocks;
- `RemoteFreeQueue`;
- `RemoteFreeDrainController`;
- `RemoteFreeQueuedByteDrainConfig`;
- `RemoteFreeQueuedByteDriftReport`.

The matrix covers four cases:

| Case | Config Shape | Expected Signal |
| --- | --- | --- |
| `matched_end_drain` | target pending 256, budget 2,621,440 bytes, capacity 256 | no drift |
| `pending_target64_budget_total` | target pending 64, budget 2,621,440 bytes, capacity 256 | pending drift only |
| `pending_target256_budget640kib` | target pending 256, budget 655,360 bytes, capacity 256 | queued-byte drift only |
| `capacity64_backpressure` | target pending 64, budget 655,360 bytes, capacity 64 | queue backpressure |

The benchmark is separate from `remote_free_mixed_size_policy.rs` so the policy
benchmark does not become a mixed-purpose file.

Updated the queued-byte budget selection note to include this experiment as a
drift diagnostics evidence source.

## Validation

Commands:

```bash
cargo fmt --all
cargo bench -p locus-alloc --bench remote_free_drift_matrix -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc queued_byte_drift_report
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Focused drift tests passed: 4 passed, 0 failed.
- Workspace tests passed: 223 unit tests and 3 `locus_alloc` doctests passed.
- Format check passed.
- Clippy passed after renaming a local `status` binding to
  `controller_status` in the new benchmark.
- The drift matrix benchmark passed and asserted every expected drift counter.

## Drift Matrix Results

Final short-run sample summaries:

```text
remote_free_drift_matrix_sample_summary=matched_end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=256 target_pending=256 queued_byte_budget=2621440 samples=8 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000 full_min=0 full_max=0 full_mean=0.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440
remote_free_drift_matrix_sample_summary=pending_target64_budget_total blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=256 target_pending=64 queued_byte_budget=2621440 samples=8 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000 full_min=0 full_max=0 full_mean=0.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440
remote_free_drift_matrix_sample_summary=pending_target256_budget640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=256 target_pending=256 queued_byte_budget=655360 samples=8 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=1966080 max_queued_bytes_over_budget_max=1966080 max_queued_bytes_over_budget_mean=1966080 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000 full_min=0 full_max=0 full_mean=0.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440
remote_free_drift_matrix_sample_summary=capacity64_backpressure blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 target_pending=64 queued_byte_budget=655360 samples=8 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=1 queue_backpressure_observed_max=1 queue_backpressure_observed_mean=1.000 full_min=3 full_max=3 full_mean=3.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360
```

Final short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_drift_matrix_matched_end_drain` | 43.019 us to 43.142 us | Performance has improved, 1 high mild outlier |
| `remote_free_drift_matrix_pending_target64_budget_total` | 42.976 us to 43.301 us | Performance has regressed, 1 high severe outlier |
| `remote_free_drift_matrix_pending_target256_budget640kib` | 42.920 us to 43.179 us | Performance has regressed, 1 high severe outlier |
| `remote_free_drift_matrix_capacity64_backpressure` | 38.630 us to 39.458 us | Performance has regressed |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

The drift report now has real-allocation evidence for:

- zero drift on a matched config;
- positive pending drift without byte drift;
- positive queued-byte drift without pending drift;
- queue backpressure without pending or byte drift.

This strengthens the diagnostic path for future adaptive remote-free policy
work because the signals are not only unit-test artifacts.

## Next Step

Use the matrix to compare candidate adaptive actions. The next policy question
is whether positive drift should trigger earlier drains, larger queue capacity,
larger drain batches, or a revised queued-byte budget.
