# Remote Free Benchmark Findings

Date: 2026-07-02

## What Exists

Locus now has a safe `RemoteFreeQueue<T>` primitive with cloneable producer sinks and owner-side bounded batch draining. It has focused tests for batching, invalid configuration, and closed-owner enqueue failures.

The benchmark suite now covers:

- persistent generic vector handoff through `RemoteFreeQueue`;
- KV block handle return through `RemoteFreeQueue`;
- KV block handle return at batch limits 8, 32, 64, 128, and 256;
- request scratch arena return through `RemoteFreeQueue`;
- allocator-specific persistent handoff baselines for mimalloc, jemalloc, and the explicit system allocator.

## Current Evidence

For generic vector handoff, `RemoteFreeQueue` beat the direct persistent channel handoff in experiment 0055:

- `remote_free_queue_persistent_handoff_256x4k`: 54.873 us to 55.169 us.
- `vec_persistent_worker_handoff_256x4k`: 71.012 us to 72.160 us.

For KV block handle release, same-thread reuse remains much faster than remote release in experiment 0056:

- `kv_remote_free_queue_release_256x4k`: 20.391 us to 20.782 us.
- `kv_block_pool_cycle_256x4k`: 1.1982 us to 1.2193 us.

For KV block handle release, larger batch sizes improved this all-at-once microbenchmark:

- batch 8: 36.920 us to 38.124 us.
- batch 32: 20.059 us to 20.257 us.
- batch 64: 14.637 us to 14.913 us.
- batch 128: 10.894 us to 11.011 us.
- batch 256: 5.5519 us to 5.7110 us.

For request scratch arena return, remote completion roughly doubled the same-thread pooled request cycle in experiment 0058:

- `request_remote_free_queue_return_16x64x256b`: 6.7133 us to 6.8108 us.
- `request_scratch_pool_cycle_16x64x256b`: 3.0811 us to 3.0954 us.

## Interpretation

Batch size is a real performance lever for this synthetic KV remote-free shape. The current results favor larger batches for throughput, but they do not prove larger batches are better for inference serving because larger batches can delay release visibility.

The current remote-free benchmarks still run on logical handles and generic vectors. They do not prove NUMA placement, cache locality, or GPU staging behavior.

## Next Questions

- What batch policy balances throughput with release latency under mixed short and long request traces?
- Should `RemoteFreeQueue` expose nonblocking enqueue or backpressure metrics for scheduler feedback?
- Should KV block pools own per-node remote-free queues directly, or should queues remain separate runtime infrastructure?
- How should remote-free draining interact with NUMA placement evidence from `numa_maps`, cgroup `memory.numa_stat`, and node `numastat`?
