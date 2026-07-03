# Best Benchmark Results So Far

Date: 2026-07-03

This note preserves the strongest benchmark results observed so far so they are easy to reuse in future allocator design, readme material, release notes, or deeper benchmark planning.

These are best observed local microbenchmark results, not final claims about production serving performance. Placement-sensitive results still require NUMA proof from `numa_maps`, cgroup `memory.numa_stat`, node `numastat`, or equivalent evidence.

## Best Results

| Area | Best observed result | Source |
| --- | --- | --- |
| Scratch arena fast path | `scratch_arena_reset_cycle_64x256b`: 200.06 ns to 201.77 ns vs `vec_allocation_cycle_64x256b`: 618.20 ns to 620.74 ns | `documentation/experiments/0001-scratch-arena-foundation.md` |
| Scratch arena vs uninitialized Vec | `scratch_arena_reset_cycle_64x256b`: 205.04 ns to 217.84 ns vs `vec_uninit_capacity_allocation_cycle_64x256b`: 605.70 ns to 608.27 ns | `documentation/experiments/0046-vec-uninit-capacity-benchmark-baseline.md` |
| Request-affine arena reuse | `request_scratch_pool_cycle_16x64x256b`: 3.1759 us to 3.1842 us vs `request_vec_allocation_cycle_16x64x256b`: 12.344 us to 12.522 us | `documentation/experiments/0010-reusable-request-scratch-pool.md` |
| KV block reuse | `kv_block_pool_cycle_256x4k`: 1.1499 us to 1.1556 us vs `kv_vec_allocation_cycle_256x4k`: 16.840 us to 16.917 us | `documentation/experiments/0011-node-tagged-kv-block-pool.md` |
| KV block reuse vs uninitialized Vec | `kv_block_pool_cycle_256x4k`: 1.1526 us to 1.1558 us vs `kv_vec_uninit_capacity_allocation_cycle_256x4k`: 5.5628 us to 5.6642 us | `documentation/experiments/0046-vec-uninit-capacity-benchmark-baseline.md` |
| Remote-free generic handoff | `remote_free_queue_persistent_handoff_256x4k`: 54.873 us to 55.169 us vs `vec_persistent_worker_handoff_256x4k`: 71.012 us to 72.160 us | `documentation/experiments/0055-remote-free-queue-benchmark.md` |
| KV remote-free large batch | `kv_remote_free_queue_release_batch256_256x4k`: 5.5519 us to 5.7110 us, fastest observed KV remote-free batch sweep point | `documentation/experiments/0059-kv-remote-free-large-batches.md` |
| Nonblocking remote-free backpressure | `remote_free_try_enqueue_backpressure_256x4k_batch64`: 53.494 us to 53.681 us, with repeated pre-sample `full_mean=0.000` | `documentation/experiments/0130-remote-free-repeated-backpressure-samples.md` |
| THP-advised mapped scratch first touch | `mapped_scratch_write_touch_4mib_hugepage_advice`: 27.359 us to 27.781 us vs default 675.72 us to 695.21 us and no-hugepage 682.37 us to 694.57 us | `documentation/experiments/0110-mapped-scratch-thp-write-touch-benchmark.md` |

## Current Interpretation

- The reusable domain allocator paths are the strongest current Locus evidence: scratch arena reuse, request scratch pool reuse, and KV block pool reuse all beat repeated Vec allocation baselines in their microbenchmarks.
- The uninitialized Vec baselines are the fairest allocator-cost comparison for arena paths that do not initialize every byte on each allocation.
- The remote-free results show that batching and owner-side draining are worth keeping in the runtime design, but latency and scheduler policy still need mixed-trace benchmarks.
- The nonblocking backpressure matrix suggests queue capacity should be tested before drain batch size when trying to reduce `full_count`.
- The THP-advised mapped scratch result is the largest single timing delta observed, but it needs repeated runs and page-size proof before it can be treated as stable allocator guidance.

## Follow-Up Use

Use this note as the shortlist for:

- README performance claims, after rerunning on a clean benchmark host;
- release notes, with explicit hardware and kernel metadata;
- allocator design prioritization;
- future benchmark dashboards;
- deciding which experiments deserve repeated, statistically stronger runs.
