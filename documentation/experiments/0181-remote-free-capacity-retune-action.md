# Experiment 0181: Remote-Free Capacity Retune Action

Date: 2026-07-03

## Postulate

[Postulate 0173](../postulates/0173-remote-free-capacity-retune-action.md)
claimed that the `increase_queue_capacity` retune hint should be tested as a
capacity action candidate against both queue backpressure and owner-side
release latency before it informs adaptive remote-free policy.

## Change

Added `remote_free_capacity_retune`, a focused benchmark target using:

- real `Vec<u8>` allocation blocks;
- `RemoteFreeQueue`;
- `RemoteFreeDrainController`;
- `RemoteFreeQueuedByteDrainConfig`;
- `RemoteFreeQueuedByteDriftReport`;
- `RemoteFreeQueuedByteRetuneHint`.

The benchmark keeps the configured target window fixed at 64 pending items and
4096 bytes per item while changing only queue capacity:

- baseline capacity 64;
- candidate capacity 128;
- candidate capacity 256.

It asserts `full_count`, forced drains, drain rounds, max pending items, max
queued bytes, over-target drift, max wait, mean wait, and retune hint for each
case.

Updated the queued-byte budget selection note with the capacity-action
tradeoff.

## Validation

Commands:

```bash
cargo fmt --all
cargo bench -p locus-alloc --bench remote_free_capacity_retune -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc queued_byte
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Focused queued-byte tests passed: 34 passed, 0 failed.
- Capacity retune benchmark passed and asserted every expected counter.
- Format check passed.
- Clippy passed with `-D warnings` after renaming a local `status` binding to
  `controller_status`.
- Workspace tests passed: 224 unit tests and 3 `locus_alloc` doctests passed.

## Capacity Retune Results

Final short-run sample summaries:

```text
remote_free_capacity_retune_sample_summary=baseline_capacity64 blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 retune_hint=increase_queue_capacity samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_capacity_retune_sample_summary=candidate_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 retune_hint=review_multiple_signals samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_pending_over_target_min=64 max_pending_over_target_max=64 max_pending_over_target_mean=64.000 max_queued_bytes_over_budget_min=262144 max_queued_bytes_over_budget_max=262144 max_queued_bytes_over_budget_mean=262144 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_capacity_retune_sample_summary=candidate_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_hint=review_multiple_signals samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
```

Final short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_capacity_retune_baseline_capacity64` | 20.240 us to 20.275 us | Performance has regressed, 2 high severe outliers |
| `remote_free_capacity_retune_candidate_capacity128` | 21.516 us to 21.604 us | Performance has regressed, 1 low mild outlier |
| `remote_free_capacity_retune_candidate_capacity256` | 22.056 us to 22.223 us | Performance has regressed, 1 low mild outlier |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

Increasing capacity from 64 to 128 reduced `full_count` from 3 to 2, but
increased max wait from 2 to 4 bursts, mean wait from 1.500 to 3.000 bursts,
and retained queued bytes from 262,144 to 524,288.

Increasing capacity from 64 to 256 removed `full_count`, but increased max
wait from 2 to 8 bursts, mean wait from 1.500 to 4.500 bursts, and retained
queued bytes from 262,144 to 1,048,576. The retune hint moved from
`increase_queue_capacity` to `review_multiple_signals` because the original
target window was exceeded.

Capacity is therefore a backpressure action, not a complete adaptive policy.
It must be paired with drain cadence or queued-byte budget validation.

## Next Step

Use the `review_multiple_signals` result to test an earlier-drain candidate at
larger capacity, keeping the same retained-byte target window.
