# Experiment 0057: KV Remote Free Batch Size

Date: 2026-07-02

## Postulate

See `documentation/postulates/0049-kv-remote-free-batch-size.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_remote_free_queue_release_batch8_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_remote_free_queue_release_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_remote_free_queue_release_batch64_256x4k
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

| Benchmark | Batch limit | Time |
| --- | ---: | ---: |
| `kv_remote_free_queue_release_batch8_256x4k` | 8 | 36.920 us to 38.124 us |
| `kv_remote_free_queue_release_256x4k` | 32 | 20.059 us to 20.257 us |
| `kv_remote_free_queue_release_batch64_256x4k` | 64 | 14.637 us to 14.913 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

## Conclusion

The postulate survived. Batch size materially changed the KV remote-free queue release measurement in this local run.

The larger batch limits reduced measured time for this 256-handle workload, with batch 64 fastest among the tested values. This suggests owner-side drain overhead and channel backpressure are meaningful, but the result should be treated as workload-specific evidence. Larger batches can increase release latency in a real scheduler even when they improve this microbenchmark.
