# Experiment 0052: Persistent Worker Handoff Benchmark

Date: 2026-07-02

## Postulate

See `documentation/postulates/0044-persistent-worker-handoff-benchmark.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 vec_persistent_worker_handoff_256x4k
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

Focused Criterion samples:

| Benchmark | Time |
| --- | ---: |
| `vec_persistent_worker_handoff_256x4k` | 70.949 us to 71.712 us |
| `vec_producer_consumer_handoff_256x4k` | 90.781 us to 91.928 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The persistent-worker benchmark starts producer and consumer threads before Criterion iterations. Each iteration sends a run command to the producer, transfers 256 zero-filled 4096-byte vectors through a bounded channel, drops them on the consumer, and waits for completion.

## Conclusion

The postulate survived. The persistent-worker handoff benchmark is lower than the spawn-per-iteration handoff baseline in this run, which confirms that the earlier benchmark included meaningful thread creation cost.

The benchmark still includes allocation, bounded-channel handoff, and cross-thread drop costs. It does not yet test a Locus remote-free batching design.
