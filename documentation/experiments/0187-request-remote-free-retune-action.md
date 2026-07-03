# Experiment 0187: Request Remote-Free Retune Action

Date: 2026-07-03

## Postulate

[Postulate 0179](../postulates/0179-request-remote-free-retune-action.md)
claimed that the request-affine arena remote-free benchmark should report
`RemoteFreeQueuedByteRetuneAction` from a queued-byte drift report while
preserving real request arena close behavior and measured queued-byte counters.

## Change

Updated `request_remote_free_policy` so every policy case carries the same
8-request, 262,144-byte queued-byte drift target.

The benchmark now records and asserts:

- max pending over-target;
- max queued bytes over-budget;
- queue backpressure observation;
- `retune_hint`;
- `retune_action`.

The workload remains unchanged:

- 16 real request arenas;
- four bursts of four requests;
- 32 KiB request arena capacity;
- queue capacity 16;
- drain batch limit 8;
- owner release stays inside `RequestScratchPool::close_request`.

## Validation

Commands:

```bash
cargo fmt --all
cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc remote_free::drift
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Request focused benchmark passed and asserted every expected drift action.
- Focused drift tests passed: 7 passed, 0 failed.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 226 unit tests and 3 `locus_alloc` doctests passed.

## Request Retune Results

Repeated request policy summaries:

```text
request_remote_free_policy_sample_summary=end_drain requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 retune_hint=review_multiple_signals retune_action=drain_earlier samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=16 max_pending_max=16 max_pending_mean=16.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=2.500 mean_wait_max=2.500 mean_wait_mean=2.500 max_pending_over_target_min=8 max_pending_over_target_max=8 max_pending_over_target_mean=8.000 max_queued_bytes_over_budget_min=262144 max_queued_bytes_over_budget_max=262144 max_queued_bytes_over_budget_mean=262144 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
request_remote_free_policy_sample_summary=max_wait2 requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
request_remote_free_policy_sample_summary=max_queued256kib requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
```

Short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 24.522 us to 24.661 us | Performance has improved |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 24.084 us to 24.402 us | Performance has improved |
| `request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b` | 23.827 us to 24.253 us | Performance has improved, 1 high mild outlier |

These timings are short-run validation context against local benchmark history.
They are not a new best-result claim.

## Interpretation

The postulate survived.

End-drain retained the full 16-request arena window: max pending 16, max
queued bytes 524,288, max wait 4 bursts, mean wait 2.500 bursts, and
`retune_action=drain_earlier`.

Both max-wait-2 and queued-byte policy cases preserved the target window:
max pending 8, max queued bytes 262,144, max wait 2 bursts, mean wait 1.500
bursts, zero queue backpressure, and `retune_action=keep_config`.

This extends retune-action evidence to the request-affine arena path while
keeping the release boundary explicit through
`RequestScratchPool::close_request`.

## Next Step

Summarize the cross-domain retune-action evidence before adding adaptive
policy changes.
