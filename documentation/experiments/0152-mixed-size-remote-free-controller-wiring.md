# Experiment 0152: Mixed-Size Remote-Free Controller Wiring

Date: 2026-07-03

## Postulate

[Postulate 0144](../postulates/0144-mixed-size-remote-free-controller-wiring.md) claimed that `RemoteFreeDrainController` should preserve the mixed-size remote-free policy behavior from experiment 0147 when it replaces benchmark-local tracker and policy glue.

## Change

The `remote_free_mixed_size_policy` benchmark now uses `RemoteFreeDrainController`.

The controller handles:

- submit accounting for each successfully enqueued mixed-size block;
- queue and tracker pending-count consistency checks;
- policy status generation;
- policy decisions;
- FIFO drain accounting.

The benchmark keeps the nonblocking enqueue loop, forced drains on full queue attempts, and `TraceBlock` release behavior explicit.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
cargo test -p locus-alloc remote_free_drain_controller
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run`: passed.
- `cargo test -p locus-alloc remote_free_drain_controller`: passed, 3 focused tests.
- `cargo test --workspace`: passed, 165 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Host Benchmark

Host mixed-size controller counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 41.546 us to 41.685 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 36.639 us to 37.419 us |

## Docker Benchmark

Docker mixed-size controller counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 110.50 us to 111.33 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 31.910 us to 32.255 us |

The Docker run printed Criterion change percentages from mounted workspace history. Those percentages are not used here because they compare across contexts. The counters are the primary behavior evidence.

## Interpretation

The postulate survived.

The controller preserved the deterministic mixed-size counters from experiment 0147 exactly. Max-wait-2 still kept `full_count` and forced drains at zero while reducing max pending blocks from 256 to 64, peak queued bytes from 2,621,440 to 655,360, max wait from 8 bursts to 2 bursts, and mean wait from 4.500 bursts to 1.500 bursts.

The host max-wait-2 timing did not beat the current best mixed-size queued-byte policy result from experiment 0146, so the best-results note was not changed. Docker timing remained favorable for max-wait-2. The important result is that all current policy benchmarks now share `RemoteFreeDrainController` while preserving their workload-specific release behavior.

## Next Step

Add a small runtime-facing owner-loop example or helper documentation that shows how a domain allocator should combine `RemoteFreeQueue`, `RemoteFreeDrainController`, and its explicit release closure.
