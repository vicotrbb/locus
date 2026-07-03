# Experiment 0128: Remote Free Backpressure Benchmark

Date: 2026-07-03

## Postulate

[Postulate 0120](../postulates/0120-remote-free-backpressure-benchmark.md) claimed that the nonblocking remote-free path should have a focused benchmark measuring both completion latency and queue-full backpressure.

## Change

Added a separate Criterion benchmark target, `remote_free_backpressure`, to `locus-alloc`.

The target includes:

- `remote_free_try_enqueue_backpressure_256x4k_batch8`;
- `remote_free_try_enqueue_backpressure_256x4k_batch64`;
- a persistent remote producer thread;
- `RemoteFreeSink::try_enqueue` with retry on `Full`;
- owner-side draining through `RemoteFreeQueue::drain_batch`;
- a pre-benchmark sample line reporting submitted, drained, pending, full, and disconnected counters.

The benchmark lives outside `scratch_arena.rs` so the existing broad allocator benchmark file does not keep growing.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
cargo bench -p locus-alloc --bench remote_free_backpressure -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all -- --check
git diff --check
rg -n "<literal em dash>" documentation crates README.md Cargo.toml Cargo.lock || true
```

## Results

- Host `cargo test -p locus-alloc`: 59 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

Criterion reported that gnuplot was unavailable and used the plotters backend.

Focused Criterion samples:

| Benchmark | Time |
| --- | ---: |
| `remote_free_try_enqueue_backpressure_256x4k_batch8` | 54.896 us to 56.655 us |
| `remote_free_try_enqueue_backpressure_256x4k_batch64` | 53.974 us to 54.344 us |

Pre-benchmark backpressure samples:

| Sample | Submitted | Drained | Pending | Full | Disconnected |
| --- | ---: | ---: | ---: | ---: | ---: |
| `batch8` | 256 | 256 | 0 | 15 | 0 |
| `batch64` | 256 | 256 | 0 | 194 | 0 |

## Conclusion

The benchmark portion of the postulate survived. Locus now has a focused benchmark target for nonblocking remote-free backpressure, and the target records both latency and queue-full retry evidence.

The expectation that the batch-64 case would show fewer full-queue retries did not survive this run. Batch64 was slightly faster in the short timing sample, but its one-run `full_count` was higher. That suggests the retry counter is sensitive to producer and owner scheduling, queue capacity, and drain cadence. Future work should separate queue capacity from drain batch size and sample retry counters over many benchmark iterations before treating `full_count` as a scheduler policy signal.
