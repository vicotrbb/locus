# Experiment 0051: Allocator Specific Handoff Benchmarks

Date: 2026-07-02

## Postulate

See `documentation/postulates/0043-allocator-specific-handoff-benchmarks.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena_mimalloc -- --sample-size 10 --warm-up-time 1 --measurement-time 1 mimalloc_vec_producer_consumer_handoff_256x4k
cargo bench -p locus-alloc --bench scratch_arena_jemalloc -- --sample-size 10 --warm-up-time 1 --measurement-time 1 jemalloc_vec_producer_consumer_handoff_256x4k
cargo bench -p locus-alloc --bench scratch_arena_system -- --sample-size 10 --warm-up-time 1 --measurement-time 1 system_vec_producer_consumer_handoff_256x4k
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
| `mimalloc_vec_producer_consumer_handoff_256x4k` | 64.717 us to 66.059 us |
| `jemalloc_vec_producer_consumer_handoff_256x4k` | 99.773 us to 100.11 us |
| `system_vec_producer_consumer_handoff_256x4k` | 93.158 us to 93.941 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

These benchmarks match the default handoff shape from experiment 0050. They spawn producer and consumer threads inside each iteration, allocate vectors on the producer thread, send them through a bounded channel, and drop them on the consumer thread.

## Conclusion

The postulate survived. Locus now has allocator-specific producer and consumer handoff baselines for mimalloc, jemalloc, and the explicit system allocator.

In this short run, mimalloc had the lowest handoff time, system allocator was close to the default handoff run from experiment 0050, and jemalloc was highest. The result should be treated as workload-specific evidence because the benchmark still includes thread spawn and channel costs.
