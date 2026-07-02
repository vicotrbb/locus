# Experiment 0059: KV Remote Free Large Batches

Date: 2026-07-02

## Postulate

See `documentation/postulates/0051-kv-remote-free-large-batches.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_remote_free_queue_release_batch128_256x4k
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 kv_remote_free_queue_release_batch256_256x4k
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
| `kv_remote_free_queue_release_batch128_256x4k` | 128 | 10.894 us to 11.011 us |
| `kv_remote_free_queue_release_batch256_256x4k` | 256 | 5.5519 us to 5.7110 us |

Criterion reported that gnuplot was unavailable and used the plotters backend.

## Conclusion

The postulate survived. Larger batches continued to improve this 256-handle KV remote-free microbenchmark, with batch 256 much faster than batch 128 and the earlier batch 64 result.

This does not mean batch 256 is universally better. It lets the producer enqueue the whole per-iteration handle set before the owner drains it, which improves throughput here but may delay release in a real scheduler. Future work should measure latency-sensitive mixed request workloads rather than only all-at-once release batches.
