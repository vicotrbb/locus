# Experiment 0048: Jemalloc Benchmark Baseline

Date: 2026-07-02

## Postulate

See `documentation/postulates/0040-jemalloc-benchmark-baseline.md`.

## Commands

```sh
cargo add tikv-jemallocator --dev -p locus-alloc
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena_jemalloc -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

Executed on 2026-07-02.

`cargo add tikv-jemallocator --dev -p locus-alloc` added `tikv-jemallocator 0.7.0` and locked its native build dependency.

`cargo fmt --all` completed successfully after removing a duplicate dependency entry introduced during manual target wiring.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Short Criterion samples from `scratch_arena_jemalloc`:

| Benchmark | Time |
| --- | ---: |
| `jemalloc_vec_allocation_cycle_64x256b` | 621.46 ns to 624.50 ns |
| `jemalloc_vec_uninit_capacity_allocation_cycle_64x256b` | 409.60 ns to 412.67 ns |
| `jemalloc_kv_vec_allocation_cycle_256x4k` | 19.212 us to 19.360 us |
| `jemalloc_kv_vec_uninit_capacity_allocation_cycle_256x4k` | 7.2667 us to 7.3276 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The new benchmark target is separate from `scratch_arena` and `scratch_arena_mimalloc`, so it does not replace the global allocator for either existing benchmark binary.

## Conclusion

The postulate survived. Locus now has a separate jemalloc baseline in addition to the default allocator and mimalloc baselines.

In this short run, jemalloc was slower than mimalloc for all four matching allocation cases. It was close to the default zero-filled small allocation sample from experiment 0046, but slower than the default uninitialized-capacity samples for the 64 by 256-byte and 256 by 4096-byte cases. The result should be treated as workload-specific evidence, not a general allocator ranking.
