# Experiment 0186: KV Remote-Free Retune Action

Date: 2026-07-03

## Postulate

[Postulate 0178](../postulates/0178-kv-remote-free-retune-action.md)
claimed that the KV remote-free policy benchmark should report
`RemoteFreeQueuedByteRetuneAction` from the same drift report used by generic
remote-free traces, while preserving real `KvBlockHandle` release behavior and
the measured queued-byte counters.

## Change

Updated `kv_remote_free_policy` so every policy case carries the same
64-block, 262,144-byte queued-byte drift target.

The benchmark now records and asserts:

- max pending over-target;
- max queued bytes over-budget;
- queue backpressure observation;
- `retune_hint`;
- `retune_action`.

The workload remains unchanged:

- 256 real `KvBlockHandle`s;
- eight bursts of 32 handles;
- 4096-byte KV blocks;
- queue capacity 256;
- drain batch limit 64;
- owner release stays inside `KvBlockPool::free`.

## Validation

Commands:

```bash
cargo fmt --all
cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc remote_free::drift
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- KV focused benchmark passed and asserted every expected drift action.
- Focused drift tests passed: 7 passed, 0 failed.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 226 unit tests and 3 `locus_alloc` doctests passed.

## KV Retune Results

Repeated KV policy summaries:

```text
kv_remote_free_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_hint=review_multiple_signals retune_action=drain_earlier samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
kv_remote_free_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
kv_remote_free_policy_sample_summary=max_queued256kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
```

Short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 45.452 us to 45.913 us | Performance has improved |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 44.558 us to 45.177 us | Performance has improved |
| `kv_remote_free_tracker_capacity256_batch64_max_queued256kib_256x4k` | 44.106 us to 44.659 us | Performance has improved |

These timings are short-run validation context against local benchmark history.
They are not a new best-result claim.

## Interpretation

The postulate survived.

End-drain retained the full 256-block window: max pending 256, max queued bytes
1,048,576, max wait 8 bursts, mean wait 4.500 bursts, and
`retune_action=drain_earlier`.

Both max-wait-2 and queued-byte policy cases preserved the target window:
max pending 64, max queued bytes 262,144, max wait 2 bursts, mean wait 1.500
bursts, zero queue backpressure, and `retune_action=keep_config`.

This extends retune-action evidence to a real KV-cache handle path while
keeping the release boundary explicit through `KvBlockPool::free`.

## Next Step

Wire the same drift-action reporting into request-affine arena remote-free
benchmarks.
