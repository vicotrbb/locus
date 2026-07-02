# Experiment 0058: Request Remote Free Queue Return

Date: 2026-07-02

## Postulate

See `documentation/postulates/0050-request-remote-free-queue-return.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 request_remote_free_queue_return_16x64x256b
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 request_scratch_pool_cycle_16x64x256b
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
| `request_remote_free_queue_return_16x64x256b` | 6.7133 us to 6.8108 us |
| `request_scratch_pool_cycle_16x64x256b` | 3.0811 us to 3.0954 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The remote-return benchmark owns the `RequestScratchPool` on the benchmark thread, opens and allocates 16 request arenas per iteration, sends request IDs to a persistent remote completion thread, enqueues those IDs into `RemoteFreeQueue`, and drains the queue on the owner thread by calling `RequestScratchPool::close_request`.

## Conclusion

The postulate survived. Locus now has a domain remote-free benchmark for request scratch arena return.

The remote-return path is slower than same-thread pooled request open, allocation, and close in this short run. That overhead is expected because it includes remote completion handoff and queue draining. The benchmark gives a baseline for request-affine runtime integration rather than an optimized final path.
