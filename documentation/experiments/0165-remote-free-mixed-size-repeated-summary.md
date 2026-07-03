# Experiment 0165: Remote-Free Mixed-Size Repeated Summary

Date: 2026-07-03

## Postulate

[Postulate 0157](../postulates/0157-remote-free-mixed-size-repeated-summary.md)
claimed that the remote-free mixed-size policy benchmark should print repeated
counter summaries for retained bytes and release latency before Criterion
timing.

## Change

Extended `crates/locus-alloc/benches/remote_free_mixed_size_policy.rs` so each
policy case now prints:

- the existing one-run `remote_free_mixed_size_policy_sample=...` line;
- a new eight-run `remote_free_mixed_size_policy_sample_summary=...` line.

The repeated summary records min, max, and mean for:

- `full_count`;
- forced drains;
- policy drains;
- drain rounds;
- max pending count;
- max queued bytes;
- max wait bursts;
- mean wait bursts.

The workload is unchanged: 256 real `Vec` allocations over eight bursts using
the existing mixed-size pattern.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1'
```

Results:

- Host focused benchmark: passed.
- Docker focused benchmark: passed.
- `cargo test -p locus-alloc`: passed, 77 tests plus 1 doc test.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 191 tests plus doc tests.

## Host Results

Repeated mixed-size summaries:

```text
remote_free_mixed_size_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 41.399 us to 41.506 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 36.264 us to 37.053 us |

## Docker Results

Repeated mixed-size summaries:

```text
remote_free_mixed_size_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 113.75 us to 114.11 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 32.400 us to 32.480 us |

Criterion printed change percentages from existing benchmark history. They are
not used here because they compare against prior runs from a different local
state or environment.

## Interpretation

The postulate survived.

The repeated summaries were stable across host and Docker for the deterministic
single-threaded mixed-size trace. End-drain retained up to 2,621,440 bytes with
max wait 8 bursts and mean wait 4.500 bursts. Max-wait-2 retained up to 655,360
bytes with max wait 2 bursts and mean wait 1.500 bursts while keeping
`full_count=0`.

The host timing did not beat the current best mixed-size queued-byte policy
result recorded in
`documentation/dev-notes/2026-07-03-best-benchmark-results.md`, so the
best-results note was not updated. The useful result is stronger repeated
counter evidence that age-bounded owner drains reduce retained bytes by 75
percent in this trace without adding producer backpressure.

## Next Step

Use this repeated retained-byte evidence to test a queued-byte threshold policy
directly against the current max-wait-2 policy. The comparison should preserve
the same mixed-size trace and report whether a byte-budget policy can hold peak
retained bytes near 655,360 without relying on burst-age turns.
