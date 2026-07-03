# Experiment 0144: Remote-Free Mixed-Size Policy

Date: 2026-07-03

## Postulate

[Postulate 0136](../postulates/0136-remote-free-mixed-size-policy.md) claimed that a latency-bounded owner drain policy can reduce peak queued remote-free bytes for mixed allocation sizes without increasing producer-visible backpressure.

## Change

Added the `remote_free_mixed_size_policy` Criterion benchmark target to `locus-alloc`.

The benchmark submits 256 real `Vec` allocations as eight bursts of 32 blocks. Allocation sizes cycle through this pattern:

```text
4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192
```

Total released bytes per iteration are 2,621,440 bytes.

Compared policies:

- `remote_free_mixed_size_trace_capacity256_batch64_end_drain`: capacity 256, batch 64, drain at the end of the trace;
- `remote_free_mixed_size_trace_capacity256_batch64_max_wait2`: capacity 256, batch 64, drain every two completed bursts.

The sample records:

- submitted and drained counts;
- `full_count`;
- forced drains caused by a full queue;
- policy drains caused by the latency bound;
- drain rounds;
- maximum pending item count;
- peak queued bytes;
- released bytes;
- maximum and mean logical wait in bursts.

## Host Validation

Command:

```bash
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Results:

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed, 153 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Host mixed-size policy counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 39.474 us to 39.847 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 36.718 us to 37.380 us |

## Docker Validation

Command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker mixed-size policy counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 113.28 us to 114.51 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 31.782 us to 36.113 us |

The Docker run printed Criterion change percentages from mounted workspace history. Those percentages are not used here because they compare across contexts. The timing intervals and counters are the evidence.

## Interpretation

The postulate survived.

With the same queue capacity and batch size, the latency-bounded policy kept `full_count` at zero while reducing peak queued bytes from 2,621,440 to 655,360. It also reduced max pending item count from 256 to 64, max wait from 8 bursts to 2 bursts, and mean wait from 4.500 bursts to 1.500 bursts.

The latency-bounded policy was faster in the host run and much faster in the Docker run. The timing result should still be treated as microbenchmark evidence, but the counter result is deterministic for this trace and directly supports a queued-byte or latency guard in future remote-free policy.

## Next Step

Move from benchmark-only policy simulation toward a reusable policy model for owner drain decisions. The next useful unit should take pending item count, queued bytes, and oldest pending burst age as inputs and return whether the owner should drain.
