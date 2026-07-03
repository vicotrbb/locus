# Experiment 0183: Remote-Free Mixed-Size Retune Action

Date: 2026-07-03

## Postulate

[Postulate 0175](../postulates/0175-remote-free-mixed-size-retune-action.md)
claimed that the capacity-plus-queued-byte retune action from the uniform
trace should also survive a heterogeneous mixed-size allocation trace.

## Change

Added `remote_free_mixed_size_capacity_retune`, a focused benchmark target
using:

- real `Vec<u8>` allocation blocks;
- the existing mixed-size trace pattern from 4096 bytes to 32768 bytes;
- `RemoteFreeQueue`;
- `RemoteFreeDrainController`;
- `RemoteFreeQueuedByteDrainConfig::from_item_sizes`;
- `RemoteFreeQueuedByteDriftReport`;
- `RemoteFreeQueuedByteRetuneHint`.

The benchmark keeps the configured target window fixed at 64 pending items and
655,360 queued bytes while testing:

- baseline capacity 64 without policy drains;
- candidate capacity 128 without policy drains;
- candidate capacity 256 without policy drains;
- policy capacity 128 with queued-byte drains;
- policy capacity 256 with queued-byte drains.

It asserts `full_count`, forced drains, policy drains, drain rounds, max
pending items, max queued bytes, over-target drift, max wait, mean wait, and
retune hint for each case.

## Validation

Commands:

```bash
cargo fmt --all
cargo bench -p locus-alloc --bench remote_free_mixed_size_capacity_retune -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Mixed-size capacity retune benchmark passed and asserted every expected
  counter.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 224 unit tests and 3 `locus_alloc` doctests passed.
- The policy-action cases used real `Vec<u8>` blocks, heterogeneous retained
  sizes, `RemoteFreeQueue`, `RemoteFreeDrainController`, queued-byte
  accounting, and actual queue drain paths.

## Mixed-Size Retune Results

Final short-run sample summaries:

```text
remote_free_mixed_size_capacity_retune_sample_summary=baseline_capacity64 blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 drain_with_policy=0 retune_hint=increase_queue_capacity samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_capacity_retune_sample_summary=candidate_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_queued_bytes_min=1310720 max_queued_bytes_max=1310720 max_queued_bytes_mean=1310720 max_pending_over_target_min=64 max_pending_over_target_max=64 max_pending_over_target_mean=64.000 max_queued_bytes_over_budget_min=655360 max_queued_bytes_over_budget_max=655360 max_queued_bytes_over_budget_mean=655360 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_mixed_size_capacity_retune_sample_summary=candidate_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=1966080 max_queued_bytes_over_budget_max=1966080 max_queued_bytes_over_budget_mean=1966080 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_capacity_retune_sample_summary=policy_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=1 retune_hint=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_capacity_retune_sample_summary=policy_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=1 retune_hint=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Final short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_mixed_size_capacity_retune_baseline_capacity64` | 37.491 us to 38.157 us | 1 high mild outlier, 1 high severe outlier |
| `remote_free_mixed_size_capacity_retune_candidate_capacity128` | 40.741 us to 40.843 us | No outliers reported |
| `remote_free_mixed_size_capacity_retune_candidate_capacity256` | 42.881 us to 42.989 us | No outliers reported |
| `remote_free_mixed_size_capacity_retune_policy_capacity128` | 37.922 us to 38.230 us | 1 low severe outlier |
| `remote_free_mixed_size_capacity_retune_policy_capacity256` | 38.114 us to 38.363 us | No outliers reported |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

Capacity 256 without policy drains removed queue backpressure, but retained
256 pending items, 2,621,440 queued bytes, max wait 8 bursts, mean wait 4.500
bursts, and returned `review_multiple_signals`.

Capacity 128 and capacity 256 with queued-byte policy drains removed queue
backpressure while preserving the heterogeneous 64-item retained-memory and
release-wait window: max pending 64, max queued bytes 655,360, max wait 2
bursts, mean wait 1.500 bursts, and `keep_config`. The action required four
owner-side policy drains, matching the mixed-size queued-byte cadence.

The uniform result from experiment 0182 was not trace-specific. The same
capacity-plus-policy action preserved the measured window when retained bytes
varied by allocation size.

## Next Step

Use the validated capacity-plus-policy action to design a small reusable
retune-action recommendation helper instead of continuing to duplicate the
decision in benchmarks.
