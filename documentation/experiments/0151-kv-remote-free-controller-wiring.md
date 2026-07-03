# Experiment 0151: KV Remote-Free Controller Wiring

Date: 2026-07-03

## Postulate

[Postulate 0143](../postulates/0143-kv-remote-free-controller-wiring.md) claimed that `RemoteFreeDrainController` should preserve the measured KV remote-free policy behavior from experiment 0148 when it replaces benchmark-local tracker and policy glue.

## Change

The `kv_remote_free_policy` benchmark now uses `RemoteFreeDrainController`.

The controller handles:

- submit accounting for each remotely completed KV handle;
- queue and tracker pending-count consistency checks;
- policy status generation;
- policy decisions;
- FIFO drain accounting.

The benchmark still keeps domain release logic explicit by calling `KvBlockPool::free` in the owner drain closure.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run
cargo test -p locus-alloc remote_free_drain_controller
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Validation results:

- `cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run`: passed.
- `cargo test -p locus-alloc remote_free_drain_controller`: passed, 3 focused tests.
- `cargo test --workspace`: passed, 165 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Host Benchmark

Host KV remote-free controller counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 0 | 0 | 4 | 256 | 1,048,576 | 1,048,576 | 8 | 4.500 |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 0 | 4 | 4 | 64 | 262,144 | 1,048,576 | 2 | 1.500 |

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 39.446 us to 40.000 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 37.709 us to 38.053 us |

## Docker Benchmark

Docker KV remote-free controller counters:

| Benchmark | Full count | Policy drains | Drain rounds | Max pending | Peak queued bytes | Released bytes | Max wait bursts | Mean wait bursts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 0 | 0 | 4 | 256 | 1,048,576 | 1,048,576 | 8 | 4.500 |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 0 | 4 | 4 | 64 | 262,144 | 1,048,576 | 2 | 1.500 |

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 92.286 us to 100.66 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 78.940 us to 99.102 us |

The Docker timing intervals were noisy. Criterion change percentages from mounted workspace history are not used here because they compare across contexts. The counters are the primary behavior evidence.

## Interpretation

The postulate survived.

The controller preserved the deterministic KV counters from experiment 0148 exactly. Max-wait-2 still kept `full_count` at zero while reducing max pending handles from 256 to 64, peak queued bytes from 1,048,576 to 262,144, max wait from 8 bursts to 2 bursts, and mean wait from 4.500 bursts to 1.500 bursts.

The host timing still favored max-wait-2 in this policy benchmark. This is not a new best KV remote-free throughput result because `kv_remote_free_queue_release_batch256_256x4k` remains much faster for pure large-batch throughput. The value here is that `RemoteFreeDrainController` now has request scratch and KV block domain evidence without hiding allocator-specific release calls.

## Next Step

Consider wiring the mixed-size policy benchmark through `RemoteFreeDrainController` or extracting a small runtime-facing example of the owner drain loop.
