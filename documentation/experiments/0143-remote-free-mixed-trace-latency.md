# Experiment 0143: Remote-Free Mixed Trace Latency

Date: 2026-07-03

## Postulate

[Postulate 0135](../postulates/0135-remote-free-mixed-trace-latency.md) claimed that increasing remote-free queue capacity can reduce nonblocking enqueue backpressure while hiding longer owner-side release latency.

## Change

Added mixed-trace benchmarks to `crates/locus-alloc/benches/remote_free_backpressure.rs`:

- `remote_free_mixed_trace_256x4k_capacity64_batch64`;
- `remote_free_mixed_trace_256x4k_capacity128_batch64`;
- `remote_free_mixed_trace_256x4k_capacity256_batch64`.

Each benchmark submits 256 real 4 KiB `Vec` allocations as eight bursts of 32 blocks. The owner drains only at the end of the eight-burst trace unless the bounded queue fills, in which case the owner performs a forced drain before retrying the enqueue.

The pre-benchmark sample records:

- submitted and drained counts;
- `full_count`;
- forced drains;
- drain rounds;
- maximum pending count;
- maximum and mean logical wait in bursts.

## Host Validation

Command:

```bash
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_backpressure -- remote_free_mixed_trace --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Results:

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed, 153 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Host mixed-trace counters:

| Benchmark | Capacity | Batch | Full count | Forced drains | Drain rounds | Max pending | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_trace_256x4k_capacity64_batch64` | 64 | 64 | 3 | 3 | 4 | 64 | 2 | 1.500 |
| `remote_free_mixed_trace_256x4k_capacity128_batch64` | 128 | 64 | 2 | 2 | 4 | 128 | 4 | 3.000 |
| `remote_free_mixed_trace_256x4k_capacity256_batch64` | 256 | 64 | 0 | 0 | 4 | 256 | 8 | 4.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_trace_256x4k_capacity64_batch64` | 17.803 us to 18.461 us |
| `remote_free_mixed_trace_256x4k_capacity128_batch64` | 18.867 us to 19.426 us |
| `remote_free_mixed_trace_256x4k_capacity256_batch64` | 19.342 us to 19.943 us |

## Docker Validation

Command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench remote_free_backpressure -- remote_free_mixed_trace --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker mixed-trace counters:

| Benchmark | Capacity | Batch | Full count | Forced drains | Drain rounds | Max pending | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_trace_256x4k_capacity64_batch64` | 64 | 64 | 3 | 3 | 4 | 64 | 2 | 1.500 |
| `remote_free_mixed_trace_256x4k_capacity128_batch64` | 128 | 64 | 2 | 2 | 4 | 128 | 4 | 3.000 |
| `remote_free_mixed_trace_256x4k_capacity256_batch64` | 256 | 64 | 0 | 0 | 4 | 256 | 8 | 4.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_trace_256x4k_capacity64_batch64` | 24.433 us to 25.060 us |
| `remote_free_mixed_trace_256x4k_capacity128_batch64` | 24.697 us to 25.187 us |
| `remote_free_mixed_trace_256x4k_capacity256_batch64` | 24.587 us to 25.096 us |

The Docker run reused Criterion history from the mounted workspace and printed change percentages. Those percentages are not used here because they compare across host and container contexts. The timing intervals and counters are the relevant evidence.

## Interpretation

The postulate survived.

Capacity 256 removed full-queue retries in both host and Docker samples, but it also retained queued releases for the longest logical interval: max wait increased to 8 bursts and mean wait increased to 4.500 bursts. Capacity 64 forced three early drains, but it kept max wait at 2 bursts and mean wait at 1.500 bursts.

The host timing also favored capacity 64 in this trace. Docker timing was much flatter across capacities, but the latency counters were identical to the host run.

This means capacity is not just a throughput or retry knob. Larger capacity can make the producer path look cleaner while increasing owner-side release delay and peak pending memory.

## Next Step

Add a memory-footprint trace that records peak queued bytes and release cadence under mixed request sizes. The next policy experiment should compare a fixed large capacity against a latency-bounded drain policy that triggers owner drains before the queue is full.
