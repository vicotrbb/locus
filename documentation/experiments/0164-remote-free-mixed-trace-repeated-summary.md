# Experiment 0164: Remote-Free Mixed Trace Repeated Summary

Date: 2026-07-03

## Postulate

[Postulate 0156](../postulates/0156-remote-free-mixed-trace-repeated-summary.md)
claimed that the remote-free mixed-trace benchmark should print repeated
pre-benchmark counter summaries, not only one sample, before Criterion timing
runs.

## Change

Extended `crates/locus-alloc/benches/remote_free_backpressure.rs` so every
mixed-trace benchmark case now prints:

- the existing one-run `remote_free_mixed_trace_sample=...` line;
- a new eight-run `remote_free_mixed_trace_sample_summary=...` line.

The repeated summary records min, max, and mean for:

- `full_count`;
- forced drains;
- drain rounds;
- max pending count;
- max wait bursts;
- mean wait bursts.

The workload is unchanged: 256 real 4 KiB `Vec` allocations submitted as eight
bursts of 32 blocks.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench remote_free_backpressure -- remote_free_mixed_trace --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench remote_free_backpressure -- remote_free_mixed_trace --sample-size 10 --warm-up-time 1 --measurement-time 1'
```

Results:

- Host focused benchmark: passed.
- Docker focused benchmark: passed.
- `cargo test -p locus-alloc`: passed, 77 tests plus 1 doc test.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 191 tests plus doc tests.

## Host Results

Repeated mixed-trace summaries:

```text
remote_free_mixed_trace_sample_summary=capacity64_batch64 blocks=256 capacity=64 batch_limit=64 samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_trace_sample_summary=capacity128_batch64 blocks=256 capacity=128 batch_limit=64 samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_mixed_trace_sample_summary=capacity256_batch64 blocks=256 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
```

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_trace_256x4k_capacity64_batch64` | 19.081 us to 19.114 us |
| `remote_free_mixed_trace_256x4k_capacity128_batch64` | 19.979 us to 20.017 us |
| `remote_free_mixed_trace_256x4k_capacity256_batch64` | 20.440 us to 20.514 us |

## Docker Results

Repeated mixed-trace summaries:

```text
remote_free_mixed_trace_sample_summary=capacity64_batch64 blocks=256 capacity=64 batch_limit=64 samples=8 full_min=3 full_max=3 full_mean=3.000 forced_drains_min=3 forced_drains_max=3 forced_drains_mean=3.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_trace_sample_summary=capacity128_batch64 blocks=256 capacity=128 batch_limit=64 samples=8 full_min=2 full_max=2 full_mean=2.000 forced_drains_min=2 forced_drains_max=2 forced_drains_mean=2.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=128 max_pending_max=128 max_pending_mean=128.000 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=3.000 mean_wait_max=3.000 mean_wait_mean=3.000
remote_free_mixed_trace_sample_summary=capacity256_batch64 blocks=256 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
```

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_trace_256x4k_capacity64_batch64` | 25.567 us to 25.622 us |
| `remote_free_mixed_trace_256x4k_capacity128_batch64` | 25.728 us to 25.767 us |
| `remote_free_mixed_trace_256x4k_capacity256_batch64` | 25.510 us to 25.542 us |

Criterion printed change percentages from existing benchmark history. They are
not used here because they compare against prior runs from a different local
state or environment.

## Interpretation

The postulate survived.

The repeated summaries were stable across host and Docker for the deterministic
single-threaded mixed trace. Capacity 64 forced three early drains and held max
wait to 2 bursts. Capacity 128 forced two early drains and held max wait to 4
bursts. Capacity 256 removed forced drains and `full_count`, but retained
remote-free work for up to 8 bursts with mean wait 4.500 bursts.

The host timing did not beat the current best mixed-trace result recorded in
`documentation/dev-notes/2026-07-03-best-benchmark-results.md`, so the
best-results note was not updated. The useful result is stronger repeated
counter evidence for the capacity versus release-latency tradeoff.

## Next Step

The next remote-free policy experiment should use repeated summaries for both
latency and retained bytes, then compare fixed high capacity against a
controller policy that drains on queued bytes before capacity becomes the only
release trigger.
