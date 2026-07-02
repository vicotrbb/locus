# Experiment 0001: Scratch Arena Foundation

Date: 2026-07-02

## Postulate

See `documentation/postulates/0001-node-tagged-scratch-arena.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 4 unit tests passed.
- `locus-core`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Benchmark command used for the recorded sample:

```sh
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Criterion reported:

- `scratch_arena_reset_cycle_64x256b`: 200.06 ns to 201.77 ns.
- `vec_allocation_cycle_64x256b`: 618.20 ns to 620.74 ns.

The scratch arena benchmark creates the backing arena once, then measures 64 aligned 256-byte allocations followed by reset across iterations. The `Vec<u8>` baseline allocates 64 separate 256-byte vectors through the default Rust allocation path.

## Conclusion

The postulate survived this first foundation check. A safe node-tagged scratch arena gives a repeatable allocation and reset benchmark, passes correctness tests, and is materially faster than the simple default-allocation baseline in this microbenchmark.

This does not prove NUMA locality. The arena currently records intended node placement but does not bind or validate physical page placement.

## Next Questions

- Should the next Linux-specific step use `mbind` on an arena backing region or begin with read-only validation through `/proc/self/numa_maps`?
- What allocator baseline should be added next: default Rust allocation of a single reusable `Vec`, libc malloc through FFI, jemalloc, mimalloc, or all of these behind separate feature flags?
