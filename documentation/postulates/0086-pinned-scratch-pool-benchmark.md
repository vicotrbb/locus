# Postulate 0086: Pinned Scratch Pool Benchmark

Date: 2026-07-02

## Statement

The pinned scratch pool should have a focused reuse-cycle benchmark against the existing scratch arena and `Vec` allocation baselines.

## Rationale

`PinnedScratchPool` introduces handle checkout, map lookup, release, and page-locked arena reuse. Unit tests prove behavior, but the allocator foundation needs timing evidence before using the pool as a staging-buffer building block.

The benchmark should measure the steady-state reuse path after one arena has already been locked. Measuring `mlock` in every iteration would mostly benchmark operating-system page-lock cost and memlock limits, not allocator reuse overhead.

This remains a host page-locked memory benchmark. It does not claim CUDA host registration, GPU-near placement, or async transfer readiness.

## Experiment

Add a Criterion benchmark named `pinned_scratch_pool_reuse_cycle_64x256b` to the existing `scratch_arena` bench target.

The benchmark should:

- create a `PinnedScratchPool`;
- lock and release one arena before measurement;
- repeatedly check out the idle locked arena;
- allocate 64 buffers of 256 bytes with 64-byte alignment;
- release the arena back to the pool;
- record pool stats through `black_box`.

Run the benchmark with the existing short sample settings and compare it to the already available `scratch_arena_reset_cycle_64x256b`, `mapped_scratch_arena_reset_cycle_64x256b`, and `vec_uninit_capacity_allocation_cycle_64x256b` baselines.

## Expected Result

The benchmark should compile under all-target checks and produce a stable timing sample on the current host when small page locks are permitted.
