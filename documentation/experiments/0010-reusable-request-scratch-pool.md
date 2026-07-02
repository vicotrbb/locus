# Experiment 0010: Reusable Request Scratch Pool

Date: 2026-07-02

## Postulate

See `documentation/postulates/0007-reusable-request-scratch-pool.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 9 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Benchmark command used for the recorded sample:

```sh
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Criterion reported:

- `scratch_arena_reset_cycle_64x256b`: 200.27 ns to 201.14 ns.
- `vec_allocation_cycle_64x256b`: 625.86 ns to 648.92 ns.
- `request_scratch_cycle_16x64x256b`: 10.611 us to 10.714 us.
- `request_vec_allocation_cycle_16x64x256b`: 12.344 us to 12.522 us.
- `request_scratch_pool_cycle_16x64x256b`: 3.1759 us to 3.1842 us.

## Conclusion

The postulate survived. Reusing closed request scratch arenas by node and capacity reduced the measured request-cycle overhead substantially in this microbenchmark.

This does not prove NUMA locality. The pool still reuses safe Vec-backed arenas tagged with intended home nodes. Linux memory-policy binding and page-placement validation remain future work.

## Next Questions

- Should idle arena capacity be bounded per node to prevent memory retention under request bursts?
- Should pooled arenas expose aggregate per-node high-water marks for scheduler feedback?
