# Experiment 0047: Mimalloc Benchmark Baseline

Date: 2026-07-02

## Postulate

See `documentation/postulates/0039-mimalloc-benchmark-baseline.md`.

## Commands

```sh
cargo add mimalloc --dev -p locus-alloc
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena_mimalloc -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

Executed on 2026-07-02.

`cargo add mimalloc --dev -p locus-alloc` added `mimalloc 0.1.52` and locked its native build dependencies.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Short Criterion samples from `scratch_arena_mimalloc`:

| Benchmark | Time |
| --- | ---: |
| `mimalloc_vec_allocation_cycle_64x256b` | 378.15 ns to 378.84 ns |
| `mimalloc_vec_uninit_capacity_allocation_cycle_64x256b` | 260.75 ns to 261.56 ns |
| `mimalloc_kv_vec_allocation_cycle_256x4k` | 17.529 us to 17.565 us |
| `mimalloc_kv_vec_uninit_capacity_allocation_cycle_256x4k` | 6.9389 us to 6.9959 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The new benchmark target is separate from `scratch_arena`, so it does not replace the default allocator for the existing benchmark binary.

## Conclusion

The postulate survived. Locus now has a first industry allocator baseline in an isolated bench binary.

The mimalloc small-allocation samples were faster than the default allocator samples recorded in experiment 0046. The mimalloc 4 KiB uninitialized-capacity sample was slower than the default allocator sample from experiment 0046 on this machine. The result should be treated as workload-specific evidence, not a general allocator ranking.
