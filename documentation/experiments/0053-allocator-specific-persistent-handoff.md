# Experiment 0053: Allocator Specific Persistent Handoff

Date: 2026-07-02

## Postulate

See `documentation/postulates/0045-allocator-specific-persistent-handoff.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena_mimalloc -- --sample-size 10 --warm-up-time 1 --measurement-time 1 mimalloc_vec_persistent_worker_handoff_256x4k
cargo bench -p locus-alloc --bench scratch_arena_jemalloc -- --sample-size 10 --warm-up-time 1 --measurement-time 1 jemalloc_vec_persistent_worker_handoff_256x4k
cargo bench -p locus-alloc --bench scratch_arena_system -- --sample-size 10 --warm-up-time 1 --measurement-time 1 system_vec_persistent_worker_handoff_256x4k
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

Focused Criterion samples:

| Benchmark | Time |
| --- | ---: |
| `mimalloc_vec_persistent_worker_handoff_256x4k` | 45.707 us to 47.076 us |
| `jemalloc_vec_persistent_worker_handoff_256x4k` | 61.359 us to 63.812 us |
| `system_vec_persistent_worker_handoff_256x4k` | 69.073 us to 70.371 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The benchmarks start producer and consumer threads before Criterion iterations. Each iteration sends a run command to the producer, transfers 256 zero-filled 4096-byte vectors through a bounded channel, drops them on the consumer, and waits for completion.

## Conclusion

The postulate survived. Locus now has persistent-worker handoff baselines for mimalloc, jemalloc, and the explicit system allocator.

All three persistent-worker measurements are lower than their spawn-per-iteration counterparts from experiment 0051. In this run, mimalloc remained the lowest, followed by jemalloc, then the explicit system allocator. These results are still microbenchmark evidence, not a complete remote-free batching design.
