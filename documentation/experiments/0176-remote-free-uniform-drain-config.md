# Experiment 0176: Remote-Free Uniform Drain Config

Date: 2026-07-03

## Postulate

[Postulate 0168](../postulates/0168-remote-free-uniform-drain-config.md)
claimed that `RemoteFreeQueuedByteDrainConfig` should support uniform retained
item shapes and should be used by the KV and request queued-byte benchmark
policy cases.

## Change

Added `RemoteFreeQueuedByteDrainConfig::from_item_shape`.

The constructor validates:

- queue capacity;
- drain batch limit;
- target pending item window;
- retained-byte budget derived from target pending items and bytes per item.

Updated the queued-byte policy cases in:

- `crates/locus-alloc/benches/kv_remote_free_policy.rs`;
- `crates/locus-alloc/benches/request_remote_free_policy.rs`.

Benchmark names, workloads, queue capacities, batch limits, and owner release
closures are unchanged.

Updated `documentation/dev-notes/2026-07-03-remote-free-budget-selection.md`
to record that `RemoteFreeQueuedByteDrainConfig` supports grouped and uniform
retained item shapes.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free_queued_byte_drain_config
cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Results:

- Focused config tests passed: 10 passed, 0 failed.
- KV and request short Criterion runs passed.
- Workspace tests passed: 214 unit tests and 3 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings`.
- Format check passed.

## KV Results

Repeated KV policy summaries after config adoption:

```text
kv_remote_free_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
kv_remote_free_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
kv_remote_free_policy_sample_summary=max_queued256kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run KV Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 69.084 us to 91.461 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 56.909 us to 73.772 us |
| `kv_remote_free_tracker_capacity256_batch64_max_queued256kib_256x4k` | 53.790 us to 88.576 us |

## Request Results

Repeated request policy summaries after config adoption:

```text
request_remote_free_policy_sample_summary=end_drain requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=16 max_pending_max=16 max_pending_mean=16.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=2.500 mean_wait_max=2.500 mean_wait_mean=2.500
request_remote_free_policy_sample_summary=max_wait2 requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
request_remote_free_policy_sample_summary=max_queued256kib requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run request Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 36.978 us to 37.160 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 36.831 us to 37.128 us |
| `request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b` | 36.149 us to 36.680 us |

## Interpretation

The postulate survived.

The uniform config constructor preserves the measured queued-byte counter
behavior in both real-allocation benchmark paths:

- KV queued-byte policy still matches max-wait-2 with peak queued bytes
  262,144, max pending blocks 64, policy drains 4, max wait 2 bursts, mean
  wait 1.500 bursts, and `full_count=0`;
- request queued-byte policy still matches max-wait-2 with peak queued bytes
  262,144, max pending requests 8, policy drains 2, max wait 2 bursts, mean
  wait 1.500 bursts, and `full_count=0`.

The short timing runs are validation context, not new best-result claims.

## Next Step

Keep heterogeneous config construction out of the API until a real call site
needs queue and batch validation for a heterogeneous retained-work shape.
