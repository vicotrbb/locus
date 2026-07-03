# Experiment 0168: KV Remote-Free Queued-Byte Policy

Date: 2026-07-03

## Postulate

[Postulate 0160](../postulates/0160-kv-remote-free-queued-byte-policy.md)
claimed that a queued-byte remote-free drain threshold can match the
max-wait-2 KV remote-free policy counters while using a budget derived directly
from real KV block size and target pending block count.

## Change

Extended `crates/locus-alloc/benches/kv_remote_free_policy.rs` with:

- a queued-byte policy case:
  `kv_remote_free_tracker_capacity256_batch64_max_queued256kib_256x4k`;
- a budget derived as `64 * 4096 = 262,144` bytes;
- eight-run sample summaries for every KV policy case.

The workload is unchanged:

- 256 real `KvBlockHandle`s;
- eight bursts of 32 handles;
- block size 4096 bytes;
- queue capacity 256;
- drain batch limit 64;
- owner release remains `KvBlockPool::free` inside the `drain_batch` closure.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench kv_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1'
```

Results:

- Host focused benchmark: passed.
- Docker focused benchmark: passed.
- `cargo test -p locus-alloc`: passed, 77 tests plus 1 doc test.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 191 tests plus doc tests.

## Host Results

Repeated KV policy summaries:

```text
kv_remote_free_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
kv_remote_free_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
kv_remote_free_policy_sample_summary=max_queued256kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 45.605 us to 53.979 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 44.156 us to 44.543 us |
| `kv_remote_free_tracker_capacity256_batch64_max_queued256kib_256x4k` | 44.291 us to 44.649 us |

## Docker Results

Repeated KV policy summaries:

```text
kv_remote_free_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=1048576 max_queued_bytes_max=1048576 max_queued_bytes_mean=1048576 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500
kv_remote_free_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
kv_remote_free_policy_sample_summary=max_queued256kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k` | 93.023 us to 110.13 us |
| `kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k` | 116.30 us to 127.20 us |
| `kv_remote_free_tracker_capacity256_batch64_max_queued256kib_256x4k` | 91.371 us to 124.05 us |

Criterion printed change percentages from existing benchmark history. They are
not used here because they compare against prior runs from a different local
state or environment.

## Interpretation

The postulate survived.

The queued-byte KV policy matched max-wait-2 exactly for repeated counters in
both host and Docker runs:

- peak queued bytes: 262,144;
- max pending count: 64;
- policy drains: 4;
- max wait: 2 bursts;
- mean wait: 1.500 bursts;
- `full_count=0`.

Compared with end-drain, the queued-byte policy reduced peak retained KV bytes
by 75 percent and reduced max wait from 8 bursts to 2 bursts without adding
producer backpressure. The owner release path still frees real `KvBlockHandle`s
through `KvBlockPool::free` inside the drain closure.

The host timing did not beat the current best KV remote-free throughput result,
so the best-results note was not updated. The important result is that the
direct byte-budget policy now has real KV handle evidence, not only generic
allocation-trace evidence.

## Next Step

Add the same queued-byte policy case to the request-scratch remote-free
benchmark or factor a tiny benchmark-local summary helper if repeated summary
formatting starts to obscure the workload logic.
