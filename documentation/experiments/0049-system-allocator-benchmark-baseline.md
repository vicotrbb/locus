# Experiment 0049: System Allocator Benchmark Baseline

Date: 2026-07-02

## Postulate

See `documentation/postulates/0041-system-allocator-benchmark-baseline.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena_system -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Short Criterion samples from `scratch_arena_system`:

| Benchmark | Time |
| --- | ---: |
| `system_vec_allocation_cycle_64x256b` | 585.33 ns to 589.16 ns |
| `system_vec_uninit_capacity_allocation_cycle_64x256b` | 583.12 ns to 587.23 ns |
| `system_kv_vec_allocation_cycle_256x4k` | 16.631 us to 16.681 us |
| `system_kv_vec_uninit_capacity_allocation_cycle_256x4k` | 5.5373 us to 5.5732 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The new benchmark target is separate from the default, mimalloc, and jemalloc benchmark binaries.

## Conclusion

The postulate survived. Locus now has an explicit system allocator benchmark target that avoids raw `malloc` and `free` calls while keeping allocator identity clear.

The system allocator samples were close to the default allocator samples from experiment 0046, which matches expectations for this local platform. The explicit target is still useful because future platforms or benchmark runners can compare default Rust allocation and `std::alloc::System` without relying on an assumption.
