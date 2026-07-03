# Experiment 0148: KV Remote-Free Drain Tracker Benchmark

Date: 2026-07-03

## Postulate

[Postulate 0140](../postulates/0140-kv-remote-free-drain-tracker-benchmark.md) claimed that `RemoteFreeDrainTracker` should preserve latency and queued-byte policy signals when used with domain KV block handles, not only with `Vec` allocation traces.

## Change

Added a focused `kv_remote_free_policy` Criterion benchmark target.

The benchmark uses:

- `KvBlockPool` with 256 blocks of 4 KiB;
- `RemoteFreeQueue<KvBlockHandle>` with capacity 256 and batch 64;
- a persistent remote completion thread;
- eight bursts of 32 KV handles;
- `RemoteFreeDrainTracker` with 4 KiB recorded per pending handle;
- `RemoteFreeDrainPolicy::new()` for end-drain;
- `RemoteFreeDrainPolicy::with_max_pending_age(2)` for max-wait-2.

This keeps the domain allocator path real: handles are allocated from `KvBlockPool`, sent through a remote completion thread, enqueued into `RemoteFreeQueue`, drained by the owner, and freed back to the pool.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run
cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run`: passed.
- `cargo test --workspace`: passed, 162 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Host Benchmark

Host KV remote-free policy counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 0 | 0 | 4 | 256 | 1,048,576 | 1,048,576 | 8 | 4.500 |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 0 | 4 | 4 | 64 | 262,144 | 1,048,576 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 38.890 us to 39.361 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 38.106 us to 39.448 us |

## Docker Benchmark

Docker KV remote-free policy counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 0 | 0 | 4 | 256 | 1,048,576 | 1,048,576 | 8 | 4.500 |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 0 | 4 | 4 | 64 | 262,144 | 1,048,576 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 85.189 us to 99.999 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 38.803 us to 89.075 us |

The Docker timing intervals were noisy, but the counters matched the host run exactly. Criterion change percentages from mounted workspace history are not used here because they compare across contexts.

## Interpretation

The postulate survived.

`RemoteFreeDrainTracker` preserved the expected policy signal on a domain KV handle path. Max-wait-2 kept `full_count` at zero while reducing max pending handles from 256 to 64, peak queued bytes from 1,048,576 to 262,144, max wait from 8 bursts to 2 bursts, and mean wait from 4.500 bursts to 1.500 bursts.

This benchmark is not a new best KV remote-free throughput result. The existing large-batch KV remote-free benchmark remains the best throughput evidence. The new value here is policy-accounting coverage on a real domain handle path.

## Next Step

Add a request-scratch remote-free policy benchmark or a small owner-loop helper that combines queue draining, tracker updates, and policy decisions for domain runtimes.
