# Experiment 0046: Vec Uninit Capacity Benchmark Baseline

Date: 2026-07-02

## Postulate

See `documentation/postulates/0038-vec-uninit-capacity-benchmark-baseline.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 vec_uninit_capacity_allocation_cycle_64x256b
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_vec_uninit_capacity_allocation_cycle_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 scratch_arena_reset_cycle_64x256b
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 vec_allocation_cycle_64x256b
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_block_pool_cycle_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_vec_allocation_cycle_256x4k
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed with no changes after the final edit.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

An initial boxed uninitialized slice approach was rejected by clippy because `Box::<[MaybeUninit<u8>]>::new_uninit_slice` is stable in Rust 1.82, while the workspace MSRV is Rust 1.80. The benchmark was changed to use `Vec::<MaybeUninit<u8>>::with_capacity`, which is MSRV-compatible and still avoids byte initialization.

Short Criterion samples:

| Benchmark | Time |
| --- | ---: |
| `scratch_arena_reset_cycle_64x256b` | 205.04 ns to 217.84 ns |
| `vec_uninit_capacity_allocation_cycle_64x256b` | 605.70 ns to 608.27 ns |
| `vec_allocation_cycle_64x256b` | 636.90 ns to 638.40 ns |
| `kv_block_pool_cycle_256x4k` | 1.1526 us to 1.1558 us |
| `kv_vec_uninit_capacity_allocation_cycle_256x4k` | 5.5628 us to 5.6642 us |
| `kv_vec_allocation_cycle_256x4k` | 16.737 us to 16.821 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

## Conclusion

The postulate survived. Uninitialized-capacity Vec allocation provides a more allocator-focused default global allocator baseline than zero-filled `Vec<u8>`, especially for 4096-byte KV block allocations. Reusable scratch and KV pools still remain substantially faster in these short samples.

The new baseline is not a jemalloc, mimalloc, or libc malloc comparison. Those remain separate industry-baseline work items.
