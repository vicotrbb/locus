# Experiment 0172: Remote-Free Uniform Benchmark Budget Helper

Date: 2026-07-03

## Postulate

[Postulate 0164](../postulates/0164-remote-free-uniform-benchmark-budget-helper.md)
claimed that the uniform queued-byte benchmark cases should derive their
retained-byte thresholds through `RemoteFreeQueuedByteBudget` instead of local
constants.

## Change

Updated the uniform queued-byte policy cases in:

- `crates/locus-alloc/benches/kv_remote_free_policy.rs`;
- `crates/locus-alloc/benches/request_remote_free_policy.rs`.

The benchmark policy constructors now use
`RemoteFreeQueuedByteBudget::from_item_shape(...).into_policy()`.

The workloads are unchanged:

- KV still returns 256 real `KvBlockHandle`s through `KvBlockPool::free`;
- request scratch still closes 16 real request arenas through
  `RequestScratchPool::close_request`;
- queue capacities, batch limits, benchmark names, and release closures are
  unchanged.

The mixed-size benchmark remains unchanged because its threshold is derived
from a heterogeneous trace pattern, not a uniform item shape.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free_queued_byte_budget
cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Results:

- Focused helper tests passed: 9 passed, 0 failed.
- Workspace tests passed: 200 unit tests and 2 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings`.
- Format check passed.
- KV and request short Criterion runs passed.

## KV Results

Repeated KV policy summaries after helper adoption:

```text
kv_remote_free_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
kv_remote_free_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
kv_remote_free_policy_sample_summary=max_queued256kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run KV Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 68.524 us to 96.982 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 52.217 us to 70.781 us |
| `kv_remote_free_tracker_capacity256_batch64_max_queued256kib_256x4k` | 54.736 us to 79.417 us |

## Request Results

Repeated request policy summaries after helper adoption:

```text
request_remote_free_policy_sample_summary=end_drain requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=16 max_pending_max=16 max_pending_mean=16.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=2.500 mean_wait_max=2.500 mean_wait_mean=2.500
request_remote_free_policy_sample_summary=max_wait2 requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
request_remote_free_policy_sample_summary=max_queued256kib requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run request Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 37.759 us to 38.112 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 36.490 us to 37.293 us |
| `request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b` | 37.096 us to 38.156 us |

## Interpretation

The postulate survived.

The helper is now validated in both uniform real-allocation benchmark paths:

- KV queued-byte policy still matches max-wait-2 counters with peak queued
  bytes 262,144, max pending blocks 64, policy drains 4, max wait 2 bursts,
  mean wait 1.500 bursts, and `full_count=0`;
- request queued-byte policy still matches max-wait-2 counters with peak queued
  bytes 262,144, max pending requests 8, policy drains 2, max wait 2 bursts,
  mean wait 1.500 bursts, and `full_count=0`.

The short timing runs are not new best-result claims. The useful result is that
`RemoteFreeQueuedByteBudget` now constructs retained-byte policies in the same
real benchmark paths that generated the queued-byte evidence.

## Next Step

Design a separate helper for heterogeneous retained work before moving the
mixed-size benchmark off its explicit trace-derived byte threshold.
