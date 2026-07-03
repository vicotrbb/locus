# Experiment 0150: Remote-Free Owner Drain Controller

Date: 2026-07-03

## Postulate

[Postulate 0142](../postulates/0142-remote-free-owner-drain-controller.md) claimed that the owner-side policy wiring around `RemoteFreeQueue`, `RemoteFreeDrainTracker`, and `RemoteFreeDrainPolicy` should become a reusable controller instead of remaining benchmark-local glue.

## Change

Added `RemoteFreeDrainController` to `locus-alloc`.

The controller:

- owns a `RemoteFreeDrainPolicy` and `RemoteFreeDrainTracker`;
- records submitted work by logical turn and queued byte size;
- records FIFO drain accounting;
- builds `RemoteFreeDrainControllerStatus` from a queue and current logical turn;
- returns the policy decision for that status;
- rejects queue and tracker pending-count drift with `RemoteFreeDrainControllerError::PendingCountMismatch`;
- leaves item-specific release logic in the domain allocator.

The request scratch remote-free policy benchmark now uses the controller for submit tracking, queue consistency checks, policy decisions, and drain accounting. The benchmark still closes request arenas explicitly through `RequestScratchPool::close_request`.

## Validation

Host commands:

```bash
cargo test -p locus-alloc remote_free_drain_controller
cargo bench -p locus-alloc --bench request_remote_free_policy --no-run
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo test -p locus-alloc remote_free_drain_controller`: passed, 3 focused tests.
- `cargo bench -p locus-alloc --bench request_remote_free_policy --no-run`: passed.
- `cargo test --workspace`: passed, 165 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Focused controller tests cover:

- policy status generation from queue stats and tracker observations;
- pending-count drift detection between queue and tracker;
- drain accounting while draining a real `RemoteFreeQueue`.

## Host Benchmark

Host request remote-free controller counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 0 | 0 | 2 | 16 | 524,288 | 524,288 | 4 | 2.500 |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 0 | 2 | 2 | 8 | 262,144 | 524,288 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 20.916 us to 21.100 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 19.138 us to 19.671 us |

## Docker Benchmark

Docker request remote-free controller counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 0 | 0 | 2 | 16 | 524,288 | 524,288 | 4 | 2.500 |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 0 | 2 | 2 | 8 | 262,144 | 524,288 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 47.274 us to 54.553 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 30.476 us to 54.687 us |

The Docker timing intervals were noisy. Criterion change percentages from mounted workspace history are not used here because they compare across contexts. The counters are the primary behavior evidence.

## Interpretation

The postulate survived.

The controller preserved the request benchmark counters from experiment 0149 exactly. Max-wait-2 still kept `full_count` at zero while reducing max pending requests from 16 to 8, peak queued arena bytes from 524,288 to 262,144, max wait from 4 bursts to 2 bursts, and mean wait from 2.500 bursts to 1.500 bursts.

The focused tests add a guard that the benchmark-local helpers did not have as a reusable API: queue and tracker pending-count drift now returns a typed error.

This is not a new best request-scratch throughput result. The value is architectural: domain allocators can share policy status and accounting control without hiding their release logic.

## Next Step

Wire the KV remote-free policy benchmark through `RemoteFreeDrainController` or add a small runtime-facing example that shows the owner loop structure for domain allocators.
