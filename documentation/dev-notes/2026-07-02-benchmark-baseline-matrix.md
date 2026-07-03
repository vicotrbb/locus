# Benchmark Baseline Matrix

Date: 2026-07-02

This note summarizes the allocator benchmark coverage currently recorded in experiment notes. The source of truth remains the individual experiment log linked in each row.

## Current Microbenchmarks

| Workload | Locus case | Baseline case | Current sample | Source |
| --- | --- | --- | --- | --- |
| Scratch reset cycle, 64 allocations of 256 bytes | `scratch_arena_reset_cycle_64x256b` | `vec_allocation_cycle_64x256b` | 200.06 ns to 201.77 ns vs 618.20 ns to 620.74 ns | `documentation/experiments/0001-scratch-arena-foundation.md` |
| Request churn, 16 requests with 64 allocations of 256 bytes | `request_scratch_pool_cycle_16x64x256b` | `request_vec_allocation_cycle_16x64x256b` | 3.1759 us to 3.1842 us vs 12.344 us to 12.522 us | `documentation/experiments/0010-reusable-request-scratch-pool.md` |
| KV block churn, 256 blocks of 4096 bytes | `kv_block_pool_cycle_256x4k` | `kv_vec_allocation_cycle_256x4k` | 1.1499 us to 1.1556 us vs 16.840 us to 16.917 us | `documentation/experiments/0011-node-tagged-kv-block-pool.md` |
| First-touch materialization, 1 MiB | `mapped_scratch_write_touch_1mib` | `vec_write_touch_1mib` | 36.140 us to 36.762 us vs 1.8701 us to 2.0037 us | `documentation/experiments/0028-first-touch-materialization-benchmark.md` |
| Scratch reset cycle, uninitialized-capacity baseline | `scratch_arena_reset_cycle_64x256b` | `vec_uninit_capacity_allocation_cycle_64x256b` | 205.04 ns to 217.84 ns vs 605.70 ns to 608.27 ns | `documentation/experiments/0046-vec-uninit-capacity-benchmark-baseline.md` |
| KV block churn, uninitialized-capacity baseline | `kv_block_pool_cycle_256x4k` | `kv_vec_uninit_capacity_allocation_cycle_256x4k` | 1.1526 us to 1.1558 us vs 5.5628 us to 5.6642 us | `documentation/experiments/0046-vec-uninit-capacity-benchmark-baseline.md` |
| Small default allocation through mimalloc | None | `mimalloc_vec_allocation_cycle_64x256b` and `mimalloc_vec_uninit_capacity_allocation_cycle_64x256b` | 378.15 ns to 378.84 ns zero-filled, 260.75 ns to 261.56 ns uninitialized capacity | `documentation/experiments/0047-mimalloc-benchmark-baseline.md` |
| KV-sized default allocation through mimalloc | None | `mimalloc_kv_vec_allocation_cycle_256x4k` and `mimalloc_kv_vec_uninit_capacity_allocation_cycle_256x4k` | 17.529 us to 17.565 us zero-filled, 6.9389 us to 6.9959 us uninitialized capacity | `documentation/experiments/0047-mimalloc-benchmark-baseline.md` |
| Small default allocation through jemalloc | None | `jemalloc_vec_allocation_cycle_64x256b` and `jemalloc_vec_uninit_capacity_allocation_cycle_64x256b` | 621.46 ns to 624.50 ns zero-filled, 409.60 ns to 412.67 ns uninitialized capacity | `documentation/experiments/0048-jemalloc-benchmark-baseline.md` |
| KV-sized default allocation through jemalloc | None | `jemalloc_kv_vec_allocation_cycle_256x4k` and `jemalloc_kv_vec_uninit_capacity_allocation_cycle_256x4k` | 19.212 us to 19.360 us zero-filled, 7.2667 us to 7.3276 us uninitialized capacity | `documentation/experiments/0048-jemalloc-benchmark-baseline.md` |
| Small default allocation through explicit system allocator | None | `system_vec_allocation_cycle_64x256b` and `system_vec_uninit_capacity_allocation_cycle_64x256b` | 585.33 ns to 589.16 ns zero-filled, 583.12 ns to 587.23 ns uninitialized capacity | `documentation/experiments/0049-system-allocator-benchmark-baseline.md` |
| KV-sized default allocation through explicit system allocator | None | `system_kv_vec_allocation_cycle_256x4k` and `system_kv_vec_uninit_capacity_allocation_cycle_256x4k` | 16.631 us to 16.681 us zero-filled, 5.5373 us to 5.5732 us uninitialized capacity | `documentation/experiments/0049-system-allocator-benchmark-baseline.md` |
| Producer and consumer handoff, 256 blocks of 4096 bytes | None | `vec_producer_consumer_handoff_256x4k` | 90.980 us to 92.328 us | `documentation/experiments/0050-producer-consumer-handoff-benchmark.md` |
| Producer and consumer handoff through mimalloc | None | `mimalloc_vec_producer_consumer_handoff_256x4k` | 64.717 us to 66.059 us | `documentation/experiments/0051-allocator-specific-handoff-benchmarks.md` |
| Producer and consumer handoff through jemalloc | None | `jemalloc_vec_producer_consumer_handoff_256x4k` | 99.773 us to 100.11 us | `documentation/experiments/0051-allocator-specific-handoff-benchmarks.md` |
| Producer and consumer handoff through explicit system allocator | None | `system_vec_producer_consumer_handoff_256x4k` | 93.158 us to 93.941 us | `documentation/experiments/0051-allocator-specific-handoff-benchmarks.md` |
| Persistent-worker producer and consumer handoff | None | `vec_persistent_worker_handoff_256x4k` | 70.949 us to 71.712 us | `documentation/experiments/0052-persistent-worker-handoff-benchmark.md` |
| Persistent-worker handoff through mimalloc | None | `mimalloc_vec_persistent_worker_handoff_256x4k` | 45.707 us to 47.076 us | `documentation/experiments/0053-allocator-specific-persistent-handoff.md` |
| Persistent-worker handoff through jemalloc | None | `jemalloc_vec_persistent_worker_handoff_256x4k` | 61.359 us to 63.812 us | `documentation/experiments/0053-allocator-specific-persistent-handoff.md` |
| Persistent-worker handoff through explicit system allocator | None | `system_vec_persistent_worker_handoff_256x4k` | 69.073 us to 70.371 us | `documentation/experiments/0053-allocator-specific-persistent-handoff.md` |
| Locus remote-free queue persistent handoff | `remote_free_queue_persistent_handoff_256x4k` | `vec_persistent_worker_handoff_256x4k` | 54.873 us to 55.169 us vs 71.012 us to 72.160 us | `documentation/experiments/0055-remote-free-queue-benchmark.md` |
| KV block remote-free queue release | `kv_remote_free_queue_release_256x4k` | `kv_block_pool_cycle_256x4k` | 20.391 us to 20.782 us vs 1.1982 us to 1.2193 us | `documentation/experiments/0056-kv-remote-free-queue-benchmark.md` |
| KV block remote-free queue batch-size sweep | `kv_remote_free_queue_release_batch8_256x4k`, `kv_remote_free_queue_release_256x4k`, `kv_remote_free_queue_release_batch64_256x4k` | None | 36.920 us to 38.124 us, 20.059 us to 20.257 us, 14.637 us to 14.913 us | `documentation/experiments/0057-kv-remote-free-batch-size.md` |
| KV block remote-free queue large-batch sweep | `kv_remote_free_queue_release_batch128_256x4k`, `kv_remote_free_queue_release_batch256_256x4k` | None | 10.894 us to 11.011 us, 5.5519 us to 5.7110 us | `documentation/experiments/0059-kv-remote-free-large-batches.md` |
| Request scratch remote-free queue return | `request_remote_free_queue_return_16x64x256b` | `request_scratch_pool_cycle_16x64x256b` | 6.7133 us to 6.8108 us vs 3.0811 us to 3.0954 us | `documentation/experiments/0058-request-remote-free-queue-return.md` |
| Nonblocking remote-free backpressure, 256 blocks of 4096 bytes | `remote_free_try_enqueue_backpressure_256x4k_batch8`, `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64`, `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8`, `remote_free_try_enqueue_backpressure_256x4k_batch64` | None | 55.116 us to 56.481 us with full count 0, 54.538 us to 57.511 us with full count 49, 53.576 us to 54.064 us with full count 0, 53.561 us to 53.986 us with full count 0 | `documentation/experiments/0129-remote-free-capacity-batch-matrix.md` |
| Mapped scratch 4 MiB write-touch with THP advice | `mapped_scratch_write_touch_4mib_hugepage_advice` | `mapped_scratch_write_touch_4mib_default` and `mapped_scratch_write_touch_4mib_no_hugepage_advice` | 27.359 us to 27.781 us vs 675.72 us to 695.21 us and 682.37 us to 694.57 us | `documentation/experiments/0110-mapped-scratch-thp-write-touch-benchmark.md` |

## Interpretation

- The safe scratch and KV reuse paths are consistently faster than repeated default allocation in these microbenchmarks.
- The uninitialized-capacity Vec baseline is a better allocator-cost baseline than zero-filled `Vec<u8>` when comparing against arena memory that is not byte-initialized on each allocation.
- The first-touch mapped arena result is intentionally slower than the default vector case because it includes mapping and page fault materialization. It should not be compared with reset-cycle fast paths.
- The short THP write-touch sample shows a large difference for `hugepage` advice in Docker, but it is baseline evidence only until repeated runs and live page-size proof confirm the mapping behavior.
- The fixed-capacity nonblocking backpressure matrix shows that queue-full retry counts are not a simple proxy for throughput. In the latest short run, capacity 64 cases were faster and reported no retries, while capacity 8 with batch 64 reported retries.
- None of these benchmark rows prove NUMA placement. Placement proof still depends on `mbind` or first-touch policy plus corroborating observability from `numa_maps`, cgroup `memory.numa_stat`, or node `numastat`.

## Missing Baselines

- Optimized domain allocator remote-free batching behavior is still missing.
- End-to-end LLM serving traces are still missing.
- Repeated THP benchmark runs tied to `mapped_scratch_thp_validation_gate=ready` evidence are still missing.

## Next Benchmarking Step

The next benchmark increment should start tying locality evidence to these domain allocator benchmarks or add a latency-sensitive mixed release workload.
