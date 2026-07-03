# Experiment 0146: Remote-Free Policy Benchmark Wiring

Date: 2026-07-03

## Postulate

[Postulate 0138](../postulates/0138-remote-free-policy-benchmark-wiring.md) claimed that the mixed-size remote-free benchmark should use `RemoteFreeDrainPolicy` directly instead of a benchmark-local policy enum.

## Change

Updated `remote_free_mixed_size_policy` so the benchmark now uses the production policy model:

- `RemoteFreeDrainPolicy::new()` for the end-drain case;
- `RemoteFreeDrainPolicy::with_max_pending_age(2)` for the max-wait-2 case;
- `RemoteFreeDrainObservation` built from queue pending count, tracked queued bytes, and oldest pending age.

The benchmark maintains a FIFO side table of pending submit bursts. This keeps `RemoteFreeQueue` encapsulated while letting the benchmark build age observations for the policy model.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
cargo test -p locus-alloc remote_free_drain_policy
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run`: passed.
- `cargo test -p locus-alloc remote_free_drain_policy`: passed, 5 tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 158 tests plus doc tests.

Host mixed-size policy counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 39.543 us to 39.945 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 35.922 us to 36.978 us |

Docker mixed-size policy counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 115.04 us to 116.19 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 31.781 us to 33.057 us |

The Docker run printed Criterion change percentages from mounted workspace history. Those percentages are not used here because they compare across contexts. The timing intervals and counters are the evidence.

## Interpretation

The postulate survived.

The production policy model reproduced the experiment 0144 counters exactly: max-wait-2 kept `full_count` at zero, reduced peak queued bytes to 655,360, reduced max pending count to 64, and reduced max wait to 2 bursts.

The host max-wait-2 interval improved versus the previous best result, from 36.718 us to 37.380 us down to 35.922 us to 36.978 us. This is a microbenchmark result, but it confirms that using the reusable policy model did not weaken the measured policy behavior.

## Next Step

Move the pending-age and queued-byte accounting pattern toward a reusable owner-loop helper. The helper should remain outside `RemoteFreeQueue` internals unless a concrete runtime integration needs queue-owned accounting.
