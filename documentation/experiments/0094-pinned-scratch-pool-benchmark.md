# Experiment 0094: Pinned Scratch Pool Benchmark

Date: 2026-07-02

## Postulate

[Postulate 0086](../postulates/0086-pinned-scratch-pool-benchmark.md) claims that the pinned scratch pool should have a focused reuse-cycle benchmark against the existing scratch arena and `Vec` allocation baselines.

## Change

Added `pinned_scratch_pool_reuse_cycle_64x256b` to the `locus-alloc` `scratch_arena` Criterion target.

The benchmark:

- creates a `PinnedScratchPool` with one arena of 32 KiB usable capacity;
- checks out and releases one arena before measurement so the measured path uses an already locked idle arena;
- checks out the arena each iteration;
- allocates 64 buffers of 256 bytes with 64-byte alignment;
- releases the arena back to the pool;
- records pool stats through `black_box`.

If small page locks are rejected by the host, the benchmark prints a skip line and does not register the benchmark.

## Commands

```text
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 scratch_arena_reset_cycle_64x256b mapped_scratch_arena_reset_cycle_64x256b vec_uninit_capacity_allocation_cycle_64x256b pinned_scratch_pool_reuse_cycle_64x256b
cargo bench -p locus-alloc --bench scratch_arena -- 64x256b --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

`cargo fmt --all` passed.

`cargo test --workspace` passed:

```text
locus-alloc: 38 passed
locus-core: 9 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 0 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

The first benchmark command failed because Criterion accepts a single filter argument, not multiple benchmark names:

```text
error: unexpected argument found
```

The corrected benchmark command passed with these short-sample timing ranges:

```text
scratch_arena_reset_cycle_64x256b
time: [200.01 ns 200.38 ns 200.82 ns]

vec_allocation_cycle_64x256b
time: [637.30 ns 650.06 ns 661.11 ns]

vec_uninit_capacity_allocation_cycle_64x256b
time: [628.49 ns 635.89 ns 645.30 ns]

mapped_scratch_arena_reset_cycle_64x256b
time: [197.75 ns 198.49 ns 198.95 ns]

pinned_scratch_pool_reuse_cycle_64x256b
time: [222.08 ns 222.52 ns 223.01 ns]
```

The same `64x256b` filter also matched request-level benchmarks:

```text
request_scratch_cycle_16x64x256b
time: [10.435 us 10.478 us 10.518 us]

request_vec_allocation_cycle_16x64x256b
time: [12.093 us 12.261 us 12.490 us]

request_scratch_pool_cycle_16x64x256b
time: [3.2234 us 3.2518 us 3.2781 us]

request_remote_free_queue_return_16x64x256b
time: [6.7183 us 6.7470 us 6.7729 us]
```

## Conclusion

The postulate survived. The pinned scratch pool now has benchmark coverage for its steady-state reuse path.

In this short local sample, pinned pool reuse was slower than direct scratch arena reuse by about 22 ns per 64 allocation cycle, which is the expected cost of checkout, handle lookup, release, and pool accounting. It remained much faster than repeated default `Vec` allocation for the same 64 by 256-byte shape.

This benchmark does not measure one-time page-lock cost, CUDA registration, GPU-near placement, or asynchronous transfer safety.
