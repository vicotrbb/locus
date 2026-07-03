# Experiment 0129: Remote Free Capacity Batch Matrix

Date: 2026-07-03

## Postulate

[Postulate 0121](../postulates/0121-remote-free-capacity-batch-matrix.md) claimed that the remote-free backpressure benchmark should separate queue capacity from owner drain batch size.

## Change

Extended `crates/locus-alloc/benches/remote_free_backpressure.rs` with the two missing off-diagonal cases:

- `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64`;
- `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8`.

The existing diagonal cases remain:

- `remote_free_try_enqueue_backpressure_256x4k_batch8`, which uses capacity 8 and batch 8;
- `remote_free_try_enqueue_backpressure_256x4k_batch64`, which uses capacity 64 and batch 64.

The pre-benchmark sample lines now print both `capacity` and `batch_limit`.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
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

Criterion reported that gnuplot was unavailable and used the plotters backend.

Focused Criterion samples:

| Benchmark | Capacity | Batch | Time |
| --- | ---: | ---: | ---: |
| `remote_free_try_enqueue_backpressure_256x4k_batch8` | 8 | 8 | 55.116 us to 56.481 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64` | 8 | 64 | 54.538 us to 57.511 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8` | 64 | 8 | 53.576 us to 54.064 us |
| `remote_free_try_enqueue_backpressure_256x4k_batch64` | 64 | 64 | 53.561 us to 53.986 us |

Pre-benchmark backpressure samples:

| Sample | Capacity | Batch | Submitted | Drained | Pending | Full | Disconnected |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `batch8` | 8 | 8 | 256 | 256 | 0 | 0 | 0 |
| `capacity8_batch64` | 8 | 64 | 256 | 256 | 0 | 49 | 0 |
| `capacity64_batch8` | 64 | 8 | 256 | 256 | 0 | 0 | 0 |
| `batch64` | 64 | 64 | 256 | 256 | 0 | 0 | 0 |

Final hygiene:

- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

## Conclusion

The postulate survived. The benchmark now separates capacity from batch size and provides a four-case matrix for nonblocking remote-free backpressure.

The data remains schedule-sensitive. In this run, capacity 64 cases had the lowest timing intervals and zero full-queue retries in the pre-benchmark samples. The capacity 8, batch 64 sample showed full-queue retries, while capacity 8, batch 8 did not in this single pre-benchmark sample. That reinforces the conclusion from experiment 0128 that `full_count` should be sampled over repeated runs before it drives scheduler policy.
