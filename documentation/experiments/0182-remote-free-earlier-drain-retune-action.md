# Experiment 0182: Remote-Free Earlier-Drain Retune Action

Date: 2026-07-03

## Postulate

[Postulate 0174](../postulates/0174-remote-free-earlier-drain-retune-action.md)
claimed that pairing larger queue capacity with an earlier queued-byte drain
trigger should remove backpressure without increasing the retained-byte window
or owner-side release wait.

## Change

Extended `remote_free_capacity_retune` with two policy-action cases:

- `policy_capacity128`;
- `policy_capacity256`.

Both cases keep the same 64-item target window and 4096-byte block size as the
capacity-only cases, but enable `RemoteFreeDrainController` with the
`RemoteFreeQueuedByteDrainConfig` policy. The benchmark now records
`drain_with_policy` and `policy_drains` in addition to full queues, forced
drains, drain rounds, pending items, queued bytes, over-target drift, max wait,
mean wait, and retune hint.

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
- Clippy passed with `-D warnings`.
- Workspace tests passed: 224 unit tests and 3 `locus_alloc` doctests passed.
- The policy-action cases used real `Vec<u8>` blocks, `RemoteFreeQueue`,
  `RemoteFreeDrainController`, queued-byte accounting, and actual queue drain
  paths.

## Earlier-Drain Results

Final short-run sample summaries:

```text
remote_free_capacity_retune_sample_summary=baseline_capacity64 blocks=256 bursts=8 burst_blocks=32 capacity=64 batch_limit=64 drain_with_policy=0 retune_hint=increase_queue_capacity samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_capacity_retune_sample_summary=candidate_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_pending_over_target_min=64 max_pending_over_target_max=64 max_pending_over_target_mean=64.000 max_queued_bytes_over_budget_min=262144 max_queued_bytes_over_budget_max=262144 max_queued_bytes_over_budget_mean=262144 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_capacity_retune_sample_summary=candidate_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=0 retune_hint=review_multiple_signals samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_capacity_retune_sample_summary=policy_capacity128 blocks=256 bursts=8 burst_blocks=32 capacity=128 batch_limit=64 drain_with_policy=1 retune_hint=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_capacity_retune_sample_summary=policy_capacity256 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drain_with_policy=1 retune_hint=keep_config samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Final short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_capacity_retune_baseline_capacity64` | 20.001 us to 20.198 us | Performance has regressed |
| `remote_free_capacity_retune_candidate_capacity128` | 21.099 us to 21.267 us | Performance has regressed, 2 high mild outliers |
| `remote_free_capacity_retune_candidate_capacity256` | 21.386 us to 21.753 us | Change within noise threshold |
| `remote_free_capacity_retune_policy_capacity128` | 19.774 us to 19.929 us | No change detected, 1 low mild outlier |
| `remote_free_capacity_retune_policy_capacity256` | 19.645 us to 19.945 us | Change within noise threshold |

These timings are short-run validation context only. They are not a new
best-result claim.

## Interpretation

The postulate survived.

Capacity 256 without policy drains removed queue backpressure, but retained
256 pending items, 1,048,576 queued bytes, max wait 8 bursts, mean wait 4.500
bursts, and returned `review_multiple_signals`.

Capacity 128 and capacity 256 with queued-byte policy drains removed queue
backpressure while preserving the capacity-64 retained-memory and release-wait
window: max pending 64, max queued bytes 262,144, max wait 2 bursts, mean wait
1.500 bursts, and `keep_config`. The only added action was four owner-side
policy drains, matching the intended queued-byte threshold cadence.

Earlier drains are therefore a stronger adaptive response than capacity alone
when the target is retained-memory and release-latency control. Capacity still
has value for producer slack, but it should be paired with an owner drain
trigger before being treated as a safe retune action.

## Next Step

Test the same capacity-plus-policy action against mixed-size remote-free traces
so the retune path is validated beyond uniform 4096-byte blocks.
