# Experiment 0056: KV Remote Free Queue Benchmark

Date: 2026-07-02

## Postulate

See `documentation/postulates/0048-kv-remote-free-queue-benchmark.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_remote_free_queue_release_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_block_pool_cycle_256x4k
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
| `kv_remote_free_queue_release_256x4k` | 20.391 us to 20.782 us |
| `kv_block_pool_cycle_256x4k` | 1.1982 us to 1.2193 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

The remote-free benchmark owns the `KvBlockPool` on the benchmark thread, allocates 256 handles per iteration, sends those handles to a persistent remote completion thread, enqueues them into `RemoteFreeQueue`, and drains the queue on the owner thread by calling `KvBlockPool::free`.

## Conclusion

The postulate survived. Locus now has a domain remote-free benchmark for KV block handle release.

The remote-free queue path is much slower than the same-thread KV block pool cycle in this short run. That is expected because it includes cross-thread handle transfer, queue enqueue, owner-side batch drain, and pool free operations. The result provides a baseline for future batching and scheduler integration work rather than a final optimization.
