# Best Benchmark Results So Far

Date: 2026-07-03

This note preserves the strongest benchmark results observed so far so they are easy to reuse in future allocator design, readme material, release notes, or deeper benchmark planning.

These are best observed local microbenchmark results, not final claims about production serving performance. Placement-sensitive results still require NUMA proof from `numa_maps`, cgroup `memory.numa_stat`, node `numastat`, or equivalent evidence.

## Best Results

| Area | Best observed result | Evidence status | Source |
| --- | --- | --- | --- |
| Scratch arena fast path | `scratch_arena_reset_cycle_64x256b`: 200.06 ns to 201.77 ns vs `vec_allocation_cycle_64x256b`: 618.20 ns to 620.74 ns | Timing benchmark with default Vec baseline | `documentation/experiments/0001-scratch-arena-foundation.md` |
| Scratch arena vs uninitialized Vec | `scratch_arena_reset_cycle_64x256b`: 205.04 ns to 217.84 ns vs `vec_uninit_capacity_allocation_cycle_64x256b`: 605.70 ns to 608.27 ns | Timing benchmark with allocator-cost baseline | `documentation/experiments/0046-vec-uninit-capacity-benchmark-baseline.md` |
| Pinned scratch locked reuse | `pinned_scratch_pool_reuse_cycle_64x256b`: 197.84 ns to 199.14 ns, fastest observed page-locked mapped arena reuse cycle | Timing benchmark plus page-lock probe coverage | `documentation/experiments/0136-pinned-scratch-pool-module.md` |
| Request-affine arena reuse | `request_scratch_pool_cycle_16x64x256b`: 3.1759 us to 3.1842 us vs `request_vec_allocation_cycle_16x64x256b`: 12.344 us to 12.522 us | Timing benchmark with request-pool reuse path | `documentation/experiments/0010-reusable-request-scratch-pool.md` |
| KV block reuse | `kv_block_pool_cycle_256x4k`: 1.1499 us to 1.1556 us vs `kv_vec_allocation_cycle_256x4k`: 16.840 us to 16.917 us | Timing benchmark with real KV block handles | `documentation/experiments/0011-node-tagged-kv-block-pool.md` |
| KV block reuse vs uninitialized Vec | `kv_block_pool_cycle_256x4k`: 1.1526 us to 1.1558 us vs `kv_vec_uninit_capacity_allocation_cycle_256x4k`: 5.5628 us to 5.6642 us | Timing benchmark with allocator-cost baseline | `documentation/experiments/0046-vec-uninit-capacity-benchmark-baseline.md` |
| Remote-free generic handoff | `remote_free_queue_persistent_handoff_256x4k`: 54.873 us to 55.169 us vs `vec_persistent_worker_handoff_256x4k`: 71.012 us to 72.160 us | Timing benchmark with persistent worker handoff | `documentation/experiments/0055-remote-free-queue-benchmark.md` |
| KV remote-free large batch | `kv_remote_free_queue_release_batch256_256x4k`: 5.5519 us to 5.7110 us, fastest observed KV remote-free batch sweep point | Timing benchmark with real KV block handles | `documentation/experiments/0059-kv-remote-free-large-batches.md` |
| Nonblocking remote-free backpressure | `remote_free_try_enqueue_backpressure_256x4k_capacity256_batch64`: 53.173 us to 53.643 us, with repeated pre-sample `full_mean=0.000`; capacity128/batch64 was close at 53.305 us to 53.598 us | Timing benchmark plus repeated queue-full samples | `documentation/experiments/0142-remote-free-large-capacity-backpressure.md` |
| Remote-free mixed trace low-latency release | `remote_free_mixed_trace_256x4k_capacity64_batch64`: 17.803 us to 18.461 us, with max wait 2 bursts and mean wait 1.500 bursts; capacity256/batch64 removed full retries but increased max wait to 8 bursts | Timing benchmark plus release-wait counters | `documentation/experiments/0143-remote-free-mixed-trace-latency.md` |
| Remote-free mixed-size queued-byte policy | `remote_free_mixed_size_trace_capacity256_batch64_max_wait2`: 35.922 us to 36.978 us vs end-drain 39.543 us to 39.945 us, with peak queued bytes reduced from 2,621,440 to 655,360 and `full_count=0` in both policies | Timing benchmark plus queued-byte and wait counters | `documentation/experiments/0146-remote-free-policy-benchmark-wiring.md` |
| THP-advised mapped scratch first touch | `mapped_scratch_write_touch_4mib_hugepage_advice`: 27.359 us to 27.781 us vs default 675.72 us to 695.21 us and no-hugepage 682.37 us to 694.57 us | Timing only for huge page adoption, requires same-run page-size proof before design use | `documentation/experiments/0110-mapped-scratch-thp-write-touch-benchmark.md` |

## Best Validation Results

| Area | Best observed validation result | Status | Source |
| --- | --- | --- | --- |
| Mapped scratch THP benchmark evidence | Docker `scratch_arena` benchmark emitted repeated compact report lines with page samples, fault samples, and Criterion timing. The run summary reported `reports=2`, `page_evidence_cohort=consistent`, `hugepage_adoption_reports=0`, and hugepage-vs-default estimate deltas from -755190000 ps to -747175000 ps | Strong negative evidence for that environment: advice was accepted for the hugepage mode, pages were touched, and the sampled mappings still used base pages | `documentation/experiments/0161-thp-report-run-summary.md` |
| Remote-free controller behavior preservation | `RemoteFreeDrainController` preserved mixed-size policy counters exactly: peak queued bytes 2,621,440 to 655,360, max pending 256 to 64, max wait 8 to 2, and `full_count=0` in both policies | Behavior-preserving runtime API evidence, not a new timing best | `documentation/experiments/0152-mixed-size-remote-free-controller-wiring.md` |

## Current Interpretation

- The reusable domain allocator paths are the strongest current Locus evidence: scratch arena reuse, pinned scratch locked reuse, request scratch pool reuse, and KV block pool reuse all beat repeated Vec allocation baselines in their microbenchmarks.
- The uninitialized Vec baselines are the fairest allocator-cost comparison for arena paths that do not initialize every byte on each allocation.
- The remote-free results show that batching and owner-side draining are worth keeping in the runtime design, but latency and scheduler policy still need mixed-trace benchmarks.
- The nonblocking backpressure matrix suggests queue capacity should be tested before drain batch size when trying to reduce `full_count`.
- The mixed-trace remote-free result shows that larger queue capacity can hide release latency, so capacity should not become a default policy knob without a release-latency or queued-byte guard.
- The mixed-size queued-byte policy result is the strongest evidence so far that owner drain policy should consider retained bytes, not only queue capacity or producer backpressure.
- The THP-advised mapped scratch result is the largest single timing delta observed, but the newest same-log Docker benchmark evidence observed base pages in the page-size samples. Treat the timing as a lead for controlled THP environments, not as current proof of huge page adoption.
- The best THP validation result is currently negative evidence: `smaps` evidence now appears directly in the benchmark log beside fault samples and Criterion timing.

## Follow-Up Use

Use this note as the shortlist for:

- README performance claims, after rerunning on a clean benchmark host;
- release notes, with explicit hardware and kernel metadata;
- allocator design prioritization;
- future benchmark dashboards;
- deciding which experiments deserve repeated, statistically stronger runs.
