# Experiment 0014: Mapped Scratch Arena

Date: 2026-07-02

## Postulate

See `documentation/postulates/0011-mapped-scratch-arena.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 18 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 2 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Benchmark command used for the recorded sample:

```sh
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Criterion reported:

- `scratch_arena_reset_cycle_64x256b`: 200.45 ns to 201.04 ns.
- `vec_allocation_cycle_64x256b`: 611.39 ns to 616.37 ns.
- `mapped_scratch_arena_reset_cycle_64x256b`: 202.26 ns to 205.60 ns.
- `request_scratch_cycle_16x64x256b`: 10.406 us to 10.458 us.
- `request_vec_allocation_cycle_16x64x256b`: 12.006 us to 12.100 us.
- `request_scratch_pool_cycle_16x64x256b`: 3.1549 us to 3.1718 us.
- `kv_block_pool_cycle_256x4k`: 1.1698 us to 1.1765 us.
- `kv_vec_allocation_cycle_256x4k`: 16.912 us to 17.031 us.
- `kv_block_table_append_release_128x16tokens`: 1.9559 us to 1.9694 us.
- `kv_vec_table_allocation_128x4k`: 8.2961 us to 8.3345 us.

## Conclusion

The postulate survived. The mapped scratch arena preserves the safe scratch API, passes correctness checks, and performs close to the Vec-backed scratch arena in this reset-cycle microbenchmark.

This still does not prove NUMA locality. The mapped arena creates an owned address range that can be used by future memory-policy and page-placement validation work.

## Next Questions

- Should the mapped scratch arena gain an optional write-touch method before NUMA policy is added?
- Should mapped arenas be used by `RequestScratchPool`, or should Vec-backed and mmap-backed request pools remain separate for benchmark clarity?
