# Experiment 0177: Remote-Free Heterogeneous Drain Config

Date: 2026-07-03

## Postulate

[Postulate 0169](../postulates/0169-remote-free-heterogeneous-drain-config.md)
claimed that `RemoteFreeQueuedByteDrainConfig` should support heterogeneous
retained item shapes and should be used by the mixed-size queued-byte benchmark
policy case.

## Change

Added `RemoteFreeQueuedByteDrainConfig::from_item_sizes`.

The constructor:

- counts retained items from the item-size iterator;
- derives retained-byte budget from the same iterator;
- rejects empty item sequences;
- rejects zero-sized retained items;
- rejects retained-byte sum overflow;
- validates queue capacity and drain batch limit against the inferred pending
  item window.

Updated the mixed-size queued-byte policy case in
`crates/locus-alloc/benches/remote_free_mixed_size_policy.rs` to use the
config. The benchmark now shares named queue-capacity and drain-batch constants
between policy construction and benchmark execution.

Updated `documentation/dev-notes/2026-07-03-remote-free-budget-selection.md`
to record that `RemoteFreeQueuedByteDrainConfig` supports grouped, uniform, and
heterogeneous retained work.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free_queued_byte_drain_config
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Results:

- Initial focused config test run failed because the test module did not import
  the private `increment_target_pending_items` helper. The helper import was
  added and the focused test was rerun.
- Focused config tests passed after the fix: 15 passed, 0 failed.
- Mixed-size short Criterion run passed.
- Format check passed.
- Workspace tests passed: 219 unit tests and 3 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings`.

## Mixed-Size Results

Repeated mixed-size policy summaries after heterogeneous config adoption:

```text
remote_free_mixed_size_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_policy_sample_summary=max_queued640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 41.342 us to 41.744 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 38.213 us to 38.696 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_queued640kib` | 38.476 us to 39.090 us |

## Interpretation

The postulate survived.

The heterogeneous config constructor preserves the measured queued-byte counter
behavior in the mixed-size allocation benchmark:

- queued-byte budget: 655,360 bytes;
- inferred target pending items: 64;
- max pending blocks: 64;
- policy drains: 4;
- max wait: 2 bursts;
- mean wait: 1.500 bursts;
- `full_count=0`.

Queued-byte config validation now covers all currently measured retained-work
shapes: grouped owner-loop, uniform KV and request benchmarks, and
heterogeneous mixed-size allocation traces.

The short timing run is validation context, not a new best-result claim.

## Next Step

Use the complete config helper set as the baseline for future adaptive
remote-free policy work instead of adding more static constructors.
