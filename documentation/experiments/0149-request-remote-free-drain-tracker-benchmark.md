# Experiment 0149: Request Remote-Free Drain Tracker Benchmark

Date: 2026-07-03

## Postulate

[Postulate 0141](../postulates/0141-request-remote-free-drain-tracker-benchmark.md) claimed that `RemoteFreeDrainTracker` should preserve remote-free policy signals for request-scratch arena returns, not only for `Vec` buffers and KV block handles.

## Change

Added a focused `request_remote_free_policy` Criterion benchmark target.

The benchmark uses:

- `RequestScratchPool` with 16 request arenas;
- 32 KiB arena capacity per request;
- 64 allocations of 256 bytes per request;
- `RemoteFreeQueue<RequestId>` with capacity 16 and batch 8;
- a persistent remote completion thread;
- four bursts of four request IDs;
- `RemoteFreeDrainTracker` with 32 KiB recorded per pending request;
- `RemoteFreeDrainPolicy::new()` for end-drain;
- `RemoteFreeDrainPolicy::with_max_pending_age(2)` for max-wait-2.

This keeps the request allocator path real: the owner opens request arenas, performs request-local scratch allocations, receives completed request IDs from a remote completion thread, drains the owner queue, and closes arenas back into `RequestScratchPool`.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench request_remote_free_policy --no-run
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test --workspace
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo bench -p locus-alloc --bench request_remote_free_policy --no-run`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 162 tests plus doc tests.

## Host Benchmark

Host request remote-free policy counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 0 | 0 | 2 | 16 | 524,288 | 524,288 | 4 | 2.500 |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 0 | 2 | 2 | 8 | 262,144 | 524,288 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 20.893 us to 21.382 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 20.127 us to 20.374 us |

## Docker Benchmark

Docker request remote-free policy counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 0 | 0 | 2 | 16 | 524,288 | 524,288 | 4 | 2.500 |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 0 | 2 | 2 | 8 | 262,144 | 524,288 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 46.344 us to 52.227 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 48.332 us to 55.056 us |

The Docker run printed Criterion change percentages from mounted workspace history. Those percentages are not used here because they compare across contexts. The timing intervals and counters are the evidence.

## Interpretation

The postulate survived.

`RemoteFreeDrainTracker` preserved the expected policy signal on request-affine scratch arena returns. Max-wait-2 kept `full_count` at zero while reducing max pending requests from 16 to 8, peak queued arena bytes from 524,288 to 262,144, max wait from 4 bursts to 2 bursts, and mean wait from 2.500 bursts to 1.500 bursts.

The host timing favored max-wait-2 slightly, while Docker timing favored end-drain slightly. The cross-environment result is therefore not a throughput claim. The important result is deterministic tracker coverage on a real request scratch allocator path.

This is not a new best request-scratch throughput result. The existing `request_remote_free_queue_return_16x64x256b` baseline remains faster. This experiment validates policy-accounting coverage for request-affine arena returns.

## Next Step

Consider extracting a small owner-side remote-free drain loop that composes `RemoteFreeQueue`, `RemoteFreeDrainTracker`, and `RemoteFreeDrainPolicy` so domain allocators can share the same policy control flow without benchmark-local wiring.
