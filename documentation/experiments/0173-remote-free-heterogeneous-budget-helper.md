# Experiment 0173: Remote-Free Heterogeneous Budget Helper

Date: 2026-07-03

## Postulate

[Postulate 0165](../postulates/0165-remote-free-heterogeneous-budget-helper.md)
claimed that `RemoteFreeQueuedByteBudget` should support heterogeneous
retained-work shapes by deriving a budget from an iterator of item sizes.

## Change

Added `RemoteFreeQueuedByteBudget::from_item_sizes`.

The constructor:

- accepts an iterator of retained item sizes;
- rejects empty item sequences;
- rejects zero-sized retained items;
- sums retained bytes with checked addition;
- returns the same typed non-zero budget used by the queued-byte drain policy.

The mixed-size queued-byte benchmark now derives its 655,360-byte budget from
the actual two-burst retained allocation-size sequence:

- 64 total pending trace blocks;
- repeated sizes from `TRACE_SIZES_U64`;
- checked conversion from burst count to target block count;
- `RemoteFreeQueuedByteBudget::from_item_sizes(...).into_policy()`.

The benchmark workload and owner release path are unchanged.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free_queued_byte_budget
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Results:

- Focused helper tests passed: 13 passed, 0 failed.
- Workspace tests passed: 204 unit tests and 2 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings`.
- Format check passed.
- The mixed-size short Criterion run passed.

## Mixed-Size Results

Repeated mixed-size policy summaries after helper adoption:

```text
remote_free_mixed_size_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_policy_sample_summary=max_queued640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 42.417 us to 42.693 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 38.819 us to 39.203 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_queued640kib` | 37.451 us to 38.587 us |

## Interpretation

The postulate survived.

`RemoteFreeQueuedByteBudget` now covers the three queued-byte policy derivation
shapes used by current remote-free experiments:

- grouped uniform shapes for the owner-loop example;
- item-count and item-size shapes for KV and request benchmarks;
- heterogeneous item-size traces for mixed-size allocation benchmarks.

The mixed-size queued-byte policy still matches max-wait-2 counters exactly:
peak queued bytes 655,360, max pending blocks 64, policy drains 4, max wait 2
bursts, mean wait 1.500 bursts, and `full_count=0`.

The short timing run is validation evidence, not a new best-result claim. The
important result is that heterogeneous retained-byte threshold construction is
now checked, typed, and exercised by a real allocation benchmark.

## Next Step

Use the budget helper family to design a small runtime configuration note for
selecting queued-byte thresholds from workload shape inputs.
