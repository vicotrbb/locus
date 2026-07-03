# Experiment 0147: Remote-Free Drain Tracker

Date: 2026-07-03

## Postulate

[Postulate 0139](../postulates/0139-remote-free-drain-tracker.md) claimed that the pending-age and queued-byte accounting used by the mixed-size remote-free benchmark should become a reusable owner-side helper instead of benchmark-local bookkeeping.

## Change

Added reusable remote-free drain accounting to `locus-alloc`:

- `RemoteFreeDrainTracker`;
- `RemoteFreeTrackedDrain`;
- `RemoteFreeDrainTrackerError`.

The tracker records successful remote-free submissions by logical turn and queued byte size. Adjacent submissions from the same turn are coalesced into one internal bucket. FIFO owner drains subtract released bytes, return the submit turn for the drained item, and report accounting errors instead of silently saturating.

The `remote_free_mixed_size_policy` benchmark now uses `RemoteFreeDrainTracker` to build `RemoteFreeDrainObservation` for `RemoteFreeDrainPolicy`.

## Validation

Host commands:

```bash
cargo test -p locus-alloc remote_free_drain_tracker
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo test -p locus-alloc remote_free_drain_tracker`: passed, 4 focused tests.
- `cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run`: passed.
- `cargo test --workspace`: passed, 162 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Focused tracker tests cover:

- policy observation from coalesced pending turns;
- empty drain rejection;
- over-release rejection;
- inconsistent final release rejection.

## Host Benchmark

Host mixed-size policy counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 42.292 us to 55.878 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 37.140 us to 37.479 us |

## Docker Benchmark

Docker mixed-size policy counters:

| Benchmark | Full count | Forced drains | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 0 | 0 | 0 | 4 | 256 | 2,621,440 | 2,621,440 | 8 | 4.500 |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 0 | 0 | 4 | 4 | 64 | 655,360 | 2,621,440 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 115.41 us to 116.72 us |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 31.398 us to 32.058 us |

The Docker run printed Criterion change percentages from mounted workspace history. Those percentages are not used here because they compare across contexts. The timing intervals and counters are the evidence.

## Interpretation

The postulate survived.

The reusable tracker preserved the deterministic mixed-size policy counters from experiments 0144 and 0146. The max-wait-2 policy still kept `full_count` at zero, reduced peak queued bytes to 655,360, reduced max pending count to 64, and reduced max wait to 2 bursts.

The host timing was slower than the previous best mixed-size policy result, so the best-results note was not changed. Docker timing remained favorable for max-wait-2. The important result for this step is that the owner-loop accounting moved out of benchmark-local ad hoc state while preserving behavior and adding invalid-accounting tests.

## Next Step

Use `RemoteFreeDrainTracker` in one domain-specific remote-free benchmark, such as KV block remote-free or request scratch remote-free, so policy accounting is tested against a domain handle path rather than only `Vec` allocations.
