# Experiment 0055: Remote Free Queue Benchmark

Date: 2026-07-02

## Postulate

See `documentation/postulates/0047-remote-free-queue-benchmark.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 remote_free_queue_persistent_handoff_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 vec_persistent_worker_handoff_256x4k
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Focused Criterion samples:

| Benchmark | Time |
| --- | ---: |
| `remote_free_queue_persistent_handoff_256x4k` | 54.873 us to 55.169 us |
| `vec_persistent_worker_handoff_256x4k` | 71.012 us to 72.160 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The queue benchmark starts one persistent producer thread before Criterion iterations. Each iteration allocates 256 zero-filled 4096-byte vectors on the producer thread, enqueues them through `RemoteFreeSink`, and drains them on the benchmark thread in batches of 32.

## Conclusion

The postulate survived. The `RemoteFreeQueue` benchmark provides a first Locus-owned remote-free batching measurement and ran faster than the direct persistent-worker channel handoff baseline in this local run.

This still releases generic vectors, not KV block handles or request arenas. The next useful step is to connect the queue shape to a domain allocator workload.
