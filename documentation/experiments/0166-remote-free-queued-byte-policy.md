# Experiment 0166: Remote-Free Queued-Byte Policy

Date: 2026-07-03

## Postulate

[Postulate 0158](../postulates/0158-remote-free-queued-byte-policy.md)
claimed that a queued-byte remote-free drain threshold can match the
retained-memory bound of the current max-wait-2 mixed-size policy without
relying on logical burst age.

## Change

Added a third policy case to
`crates/locus-alloc/benches/remote_free_mixed_size_policy.rs`:

```text
remote_free_mixed_size_trace_capacity256_batch64_max_queued640kib
```

The case uses:

```text
RemoteFreeDrainPolicy::with_max_queued_bytes(655360)
```

The workload is unchanged:

- 256 real `Vec` allocations;
- eight bursts of 32 blocks;
- existing mixed-size allocation pattern;
- capacity 256;
- batch limit 64.

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
remote_free_mixed_size_policy_sample_summary=max_queued640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 41.910 us to 42.163 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 38.400 us to 38.588 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_queued640kib` | 38.337 us to 38.626 us |

## Docker Results

Repeated mixed-size summaries:

```text
remote_free_mixed_size_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
remote_free_mixed_size_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
remote_free_mixed_size_policy_sample_summary=max_queued640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 120.26 us to 120.75 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 33.063 us to 33.151 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_queued640kib` | 32.948 us to 33.011 us |

Criterion printed change percentages from existing benchmark history. They are
not used here because they compare against prior runs from a different local
state or environment.

## Interpretation

The postulate survived.

The queued-byte policy matched max-wait-2 exactly for the repeated retained-byte
and release-latency counters in both host and Docker runs:

- peak queued bytes: 655,360;
- max pending count: 64;
- policy drains: 4;
- max wait: 2 bursts;
- mean wait: 1.500 bursts;
- `full_count=0`.

Compared with end-drain, the queued-byte policy reduced peak retained bytes by
75 percent and reduced max wait from 8 bursts to 2 bursts without adding
producer backpressure.

The host timing did not beat the current best mixed-size queued-byte policy
result recorded in
`documentation/dev-notes/2026-07-03-best-benchmark-results.md`, so the
best-results note was not updated. The important result is that the direct
byte-budget policy now has real mixed-size allocation evidence and does not need
burst-age turns to reach the same retention bound on this trace.

## Next Step

Move this queued-byte policy candidate toward a reusable owner-loop integration
example. The next step should keep allocator-specific release logic explicit
while showing how request or KV runtimes can select a byte budget from block
size, request concurrency, and target retained-memory envelope.
