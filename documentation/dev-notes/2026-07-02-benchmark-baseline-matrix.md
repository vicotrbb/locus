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

## Interpretation

- The safe scratch and KV reuse paths are consistently faster than repeated default allocation in these microbenchmarks.
- The uninitialized-capacity Vec baseline is a better allocator-cost baseline than zero-filled `Vec<u8>` when comparing against arena memory that is not byte-initialized on each allocation.
- The first-touch mapped arena result is intentionally slower than the default vector case because it includes mapping and page fault materialization. It should not be compared with reset-cycle fast paths.
- None of these benchmark rows prove NUMA placement. Placement proof still depends on `mbind` or first-touch policy plus corroborating observability from `numa_maps`, cgroup `memory.numa_stat`, or node `numastat`.

## Missing Baselines

- jemalloc comparison is still missing.
- A libc malloc baseline is still missing.
- Multithreaded producer and consumer churn is still missing.
- Remote-free or cross-thread release behavior is still missing.
- End-to-end LLM serving traces are still missing.

## Next Benchmarking Step

The next benchmark increment should add a jemalloc or libc malloc baseline behind an isolated benchmark configuration, then rerun the scratch, request, and KV churn cases with the same short-sample command shape used in the experiment notes.
