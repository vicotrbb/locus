# Experiment 0007: Request-Scoped Scratch Arenas

Date: 2026-07-02

## Postulate

See `documentation/postulates/0005-request-scoped-scratch.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 7 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Benchmark command used for the recorded sample:

```sh
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Criterion reported:

- `scratch_arena_reset_cycle_64x256b`: 199.28 ns to 202.30 ns.
- `vec_allocation_cycle_64x256b`: 612.45 ns to 615.91 ns.
- `request_scratch_cycle_16x64x256b`: 10.911 us to 11.003 us.
- `request_vec_allocation_cycle_16x64x256b`: 11.815 us to 11.894 us.

The request scratch benchmark opens 16 request arenas, performs 64 aligned 256-byte allocations per request, and closes each request arena in the timed loop. The baseline allocates 16 groups of 64 separate 256-byte vectors through default Rust allocation.

## Conclusion

The postulate survived as an API and accounting foundation. Request-scoped scratch allocation is explicit, safe, tested, and benchmarked against a request-shaped default allocation baseline.

The measured speedup is modest in this first benchmark because arena creation is included in the timed loop. That is useful evidence: request arena reuse or per-node free lists should be measured before claiming stronger allocator wins.

## Next Questions

- Should request arenas be reused from a per-node free list before Linux NUMA binding is added?
- What baseline best represents request-scoped default allocation: many request-local `Vec`s or a reusable per-request `Vec`?
