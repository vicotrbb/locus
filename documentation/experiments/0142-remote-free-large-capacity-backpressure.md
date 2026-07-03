# Experiment 0142: Remote-Free Large Capacity Backpressure

Date: 2026-07-03

Postulate: `documentation/postulates/0134-remote-free-large-capacity-backpressure.md`

## Question

Does increasing `RemoteFreeQueue` capacity beyond 64 entries reduce full-queue retries or improve timing for the 256 by 4 KiB nonblocking remote-free backpressure workload?

## Change

Extended `crates/locus-alloc/benches/remote_free_backpressure.rs` with two new Criterion cases:

- `remote_free_try_enqueue_backpressure_256x4k_capacity128_batch64`;
- `remote_free_try_enqueue_backpressure_256x4k_capacity256_batch64`.

Both cases reuse the existing backpressure benchmark body and print the same one-run sample and eight-run summary lines as the existing cases.

## Validation

Commands:

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_backpressure -- --sample-size 10 --warm-up-time 1 --measurement-time 1
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench remote_free_backpressure -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Results:

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed, 153 unit tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Host Criterion benchmark: passed and produced a new best observed nonblocking backpressure interval.
- Docker Criterion benchmark: passed and produced parseable sample and timing lines, but timings were much noisier than the host run.

## Host Results

Focused Criterion timings:

| Benchmark | Capacity | Batch | Time |
| --- | ---: | ---: | ---: |
| `remote_free_try_enqueue_backpressure_256x4k_batch8` | 8 | 8 | 54.651 us to 55.046 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64` | 8 | 64 | 54.800 us to 55.777 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8` | 64 | 8 | 53.799 us to 53.920 us |
| `remote_free_try_enqueue_backpressure_256x4k_batch64` | 64 | 64 | 53.636 us to 53.862 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity128_batch64` | 128 | 64 | 53.305 us to 53.598 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity256_batch64` | 256 | 64 | 53.173 us to 53.643 us |

Repeated pre-benchmark summaries:

| Sample | Capacity | Batch | Full min | Full max | Full mean | Pending min | Pending max | Pending mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `batch8` | 8 | 8 | 0 | 39 | 6.500 | 0 | 0 | 0.000 |
| `capacity8_batch64` | 8 | 64 | 0 | 9 | 3.500 | 0 | 0 | 0.000 |
| `capacity64_batch8` | 64 | 8 | 0 | 7 | 1.000 | 0 | 0 | 0.000 |
| `batch64` | 64 | 64 | 0 | 0 | 0.000 | 0 | 0 | 0.000 |
| `capacity128_batch64` | 128 | 64 | 0 | 0 | 0.000 | 0 | 0 | 0.000 |
| `capacity256_batch64` | 256 | 64 | 0 | 0 | 0.000 | 0 | 0 | 0.000 |

## Docker Results

Focused Criterion timings:

| Benchmark | Capacity | Batch | Time |
| --- | ---: | ---: | ---: |
| `remote_free_try_enqueue_backpressure_256x4k_batch8` | 8 | 8 | 115.43 us to 163.94 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64` | 8 | 64 | 95.555 us to 116.77 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8` | 64 | 8 | 74.472 us to 107.90 us |
| `remote_free_try_enqueue_backpressure_256x4k_batch64` | 64 | 64 | 70.802 us to 126.56 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity128_batch64` | 128 | 64 | 70.605 us to 94.424 us |
| `remote_free_try_enqueue_backpressure_256x4k_capacity256_batch64` | 256 | 64 | 83.187 us to 111.16 us |

Repeated pre-benchmark summaries:

| Sample | Capacity | Batch | Full min | Full max | Full mean | Pending min | Pending max | Pending mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `batch8` | 8 | 8 | 0 | 138 | 62.250 | 0 | 0 | 0.000 |
| `capacity8_batch64` | 8 | 64 | 0 | 59 | 13.500 | 0 | 0 | 0.000 |
| `capacity64_batch8` | 64 | 8 | 0 | 23 | 2.875 | 0 | 0 | 0.000 |
| `batch64` | 64 | 64 | 0 | 0 | 0.000 | 0 | 0 | 0.000 |
| `capacity128_batch64` | 128 | 64 | 0 | 1 | 0.375 | 0 | 0 | 0.000 |
| `capacity256_batch64` | 256 | 64 | 0 | 0 | 0.000 | 0 | 0 | 0.000 |

## Interpretation

The postulate mostly survived.

Increasing capacity beyond 64 did not reduce retries below zero because capacity64/batch64 already had zero repeated full-queue retries in the host run. It did improve the best observed host timing interval: capacity256/batch64 reached 53.173 us to 53.643 us, and capacity128/batch64 was close at 53.305 us to 53.598 us.

The Docker run supports the retry-count side of the postulate but not a stable timing conclusion. Docker timings were much noisier. Capacity128/batch64 was the best Docker interval among the larger-capacity cases and had only one retry across eight repeated samples. Capacity256/batch64 had zero retries but slower Docker timing than capacity128/batch64.

## Follow-Up

The best-results note was updated to record the new host best result.

Capacity64/batch64 still appears sufficient to eliminate full retries for this workload, while capacity128 and capacity256 bound the possible throughput benefit from extra buffering. A future mixed-trace benchmark should measure release latency and memory footprint before using larger queue capacities as a default runtime policy.
