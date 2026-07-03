# Experiment 0130: Remote Free Repeated Backpressure Samples

Date: 2026-07-03

## Postulate

[Postulate 0122](../postulates/0122-remote-free-repeated-backpressure-samples.md) claimed that the remote-free backpressure benchmark should print repeated counter samples instead of relying on a single `full_count` observation.

## Change

Updated `crates/locus-alloc/benches/remote_free_backpressure.rs` so each benchmark case prints:

- the existing one-run sample line;
- `remote_free_backpressure_sample_summary=...`;
- eight repeated counter samples summarized as min, max, and mean full-queue retries;
- min, max, and mean pending counts.

The Criterion benchmark bodies stayed unchanged, so timing comparisons remain comparable with experiment 0129.

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
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed after replacing floating-point mean formatting with fixed-point integer formatting.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.

Criterion reported that gnuplot was unavailable and used the plotters backend.

Focused Criterion samples:

| Benchmark | Capacity | Batch | Time |
| --- | ---: | ---: | ---: |
| `remote_free_try_enqueue_backpressure_256x4k_batch8` | 8 | 8 | 54.362 us to 55.330 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64` | 8 | 64 | 54.628 us to 57.192 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8` | 64 | 8 | 53.659 us to 54.253 us |
| `remote_free_try_enqueue_backpressure_256x4k_batch64` | 64 | 64 | 53.494 us to 53.681 us |

One-run pre-benchmark samples:

| Sample | Capacity | Batch | Submitted | Drained | Pending | Full | Disconnected |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `batch8` | 8 | 8 | 256 | 256 | 0 | 0 | 0 |
| `capacity8_batch64` | 8 | 64 | 256 | 256 | 0 | 20 | 0 |
| `capacity64_batch8` | 64 | 8 | 256 | 256 | 0 | 1 | 0 |
| `batch64` | 64 | 64 | 256 | 256 | 0 | 0 | 0 |

Repeated pre-benchmark summaries:

| Sample | Capacity | Batch | Runs | Full min | Full max | Full mean | Pending min | Pending max | Pending mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `batch8` | 8 | 8 | 8 | 0 | 357 | 44.625 | 0 | 0 | 0.000 |
| `capacity8_batch64` | 8 | 64 | 8 | 1 | 16 | 7.125 | 0 | 0 | 0.000 |
| `capacity64_batch8` | 64 | 8 | 8 | 0 | 7 | 0.875 | 0 | 0 | 0.000 |
| `batch64` | 64 | 64 | 8 | 0 | 0 | 0.000 | 0 | 0 | 0.000 |

Final hygiene:

- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

## Conclusion

The postulate survived. The benchmark now reports repeated counter summaries for all four capacity and batch cases while preserving the timing benchmark body.

The repeated samples make the schedule sensitivity visible instead of hiding it. In this run, capacity64/batch64 had zero retries across all eight repeated samples and the lowest timing interval. Capacity64/batch8 had low retry counts. Capacity8 cases had higher and more variable retry counts, especially capacity8/batch8 with one high-retry sample. This is stronger evidence that capacity is the first knob to test before using `full_count` as a scheduler policy signal.
