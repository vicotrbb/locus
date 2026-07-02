# Experiment 0050: Producer Consumer Handoff Benchmark

Date: 2026-07-02

## Postulate

See `documentation/postulates/0042-producer-consumer-handoff-benchmark.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 vec_producer_consumer_handoff_256x4k
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

Focused Criterion sample:

| Benchmark | Time |
| --- | ---: |
| `vec_producer_consumer_handoff_256x4k` | 90.980 us to 92.328 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The benchmark spawns the producer and consumer threads inside each iteration. The result therefore includes thread spawn, bounded-channel handoff, allocation, and cross-thread drop costs.

## Conclusion

The postulate survived. Locus now has a first cross-thread producer and consumer handoff baseline for remote-free research.

The benchmark is intentionally conservative and not yet allocator-specific across mimalloc, jemalloc, and system targets. The next refinement should reuse worker threads across iterations or add matching handoff targets for the isolated allocator benchmark binaries.
