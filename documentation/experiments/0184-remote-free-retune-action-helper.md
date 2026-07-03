# Experiment 0184: Remote-Free Retune Action Helper

Date: 2026-07-03

## Postulate

[Postulate 0176](../postulates/0176-remote-free-retune-action-helper.md)
claimed that `RemoteFreeQueuedByteDriftReport` should expose a typed
retune-action helper that maps observed drift signals to the next benchmark
candidate without duplicating ad hoc decision logic in benchmark files.

## Change

Added `RemoteFreeQueuedByteRetuneAction` with stable labels:

- `keep_config`;
- `increase_queue_capacity`;
- `drain_earlier`;
- `review_queued_byte_budget`;
- `increase_queue_capacity_and_drain_earlier`.

Added `RemoteFreeQueuedByteDriftReport::retune_action()`. The action remains
diagnostic and non-mutating. It recommends what to benchmark next from the
same pending-item, queued-byte, and queue-backpressure signals used by
`retune_hint()`.

Updated the uniform and mixed-size capacity retune benchmarks to print and
assert `retune_action` against real allocation traces.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc queued_byte_retune -- --nocapture
cargo bench -p locus-alloc --bench remote_free_capacity_retune -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo bench -p locus-alloc --bench remote_free_mixed_size_capacity_retune -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc remote_free::drift
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Focused retune-label tests passed: 3 passed, 0 failed.
- Uniform capacity retune benchmark passed and asserted every expected
  `retune_action`.
- Mixed-size capacity retune benchmark passed and asserted every expected
  `retune_action`.
- Initial full validation failed at clippy because `retune_action()` used
  unnested or-patterns. The match patterns were nested and validation was
  rerun.
- Focused drift tests passed after the fix: 7 passed, 0 failed.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 226 unit tests and 3 `locus_alloc` doctests passed.

## Retune Action Results

Uniform capacity retune summaries:

```text
remote_free_capacity_retune_sample_summary=baseline_capacity64 blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 drain_with_policy=0 retune_hint=increase_queue_capacity retune_action=increase_queue_capacity samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_capacity_retune_sample_summary=candidate_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals retune_action=increase_queue_capacity_and_drain_earlier samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_pending_over_target_min=64 max_pending_over_target_max=64 max_pending_over_target_mean=64.000 max_queued_bytes_over_budget_min=262144 max_queued_bytes_over_budget_max=262144 max_queued_bytes_over_budget_mean=262144 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_capacity_retune_sample_summary=candidate_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals retune_action=drain_earlier samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_capacity_retune_sample_summary=policy_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=1 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_capacity_retune_sample_summary=policy_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=1 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Mixed-size capacity retune summaries:

```text
remote_free_mixed_size_capacity_retune_sample_summary=baseline_capacity64 blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 drain_with_policy=0 retune_hint=increase_queue_capacity retune_action=increase_queue_capacity samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_capacity_retune_sample_summary=candidate_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals retune_action=increase_queue_capacity_and_drain_earlier samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_queued_bytes_min=1310720 max_queued_bytes_max=1310720 max_queued_bytes_mean=1310720 max_pending_over_target_min=64 max_pending_over_target_max=64 max_pending_over_target_mean=64.000 max_queued_bytes_over_budget_min=655360 max_queued_bytes_over_budget_max=655360 max_queued_bytes_over_budget_mean=655360 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_mixed_size_capacity_retune_sample_summary=candidate_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals retune_action=drain_earlier samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=1966080 max_queued_bytes_over_budget_max=1966080 max_queued_bytes_over_budget_mean=1966080 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_capacity_retune_sample_summary=policy_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=1 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_capacity_retune_sample_summary=policy_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=1 retune_hint=keep_config retune_action=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run timings are validation context only and are not a new best-result
claim.

## Interpretation

The postulate survived.

The helper distinguished the action that the recent experiments discovered:

- baseline capacity 64 with queue backpressure but no retained-window drift
  recommended `increase_queue_capacity`;
- capacity 128 with remaining backpressure and retained-window drift
  recommended `increase_queue_capacity_and_drain_earlier`;
- capacity 256 with no backpressure but retained-window drift recommended
  `drain_earlier`;
- capacity-plus-policy cases with no drift recommended `keep_config`.

Those classifications matched both uniform and heterogeneous real allocation
traces. Benchmarks no longer need to infer a retune action separately from the
drift report.

## Next Step

Use `RemoteFreeQueuedByteRetuneAction` from a runtime-facing owner-loop example
so application code can log the same recommendation that benchmarks assert.
