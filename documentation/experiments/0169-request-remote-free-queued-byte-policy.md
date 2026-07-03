# Experiment 0169: Request Remote-Free Queued-Byte Policy

Date: 2026-07-03

## Postulate

[Postulate 0161](../postulates/0161-request-remote-free-queued-byte-policy.md)
claimed that a queued-byte remote-free drain threshold can match the
max-wait-2 request remote-free policy counters while using a budget derived
from arena capacity and target pending request count.

## Change

Extended `crates/locus-alloc/benches/request_remote_free_policy.rs` with:

- a queued-byte policy case:
  `request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b`;
- a budget derived as `8 * 32768 = 262,144` bytes;
- eight-run sample summaries for every request policy case.

The workload is unchanged:

- 16 request arenas;
- four bursts of four request IDs;
- 32 KiB arena capacity;
- 64 allocations of 256 bytes per request;
- queue capacity 16;
- drain batch limit 8;
- owner release remains `RequestScratchPool::close_request` inside the
  `drain_batch` closure.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench request_remote_free_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1'
```

Results:

- Host focused benchmark: passed.
- Docker focused benchmark: passed.
- `cargo test -p locus-alloc`: passed, 77 tests plus 1 doc test.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 191 tests plus doc tests.

## Host Results

Repeated request policy summaries:

```text
request_remote_free_policy_sample_summary=end_drain requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=16 max_pending_max=16 max_pending_mean=16.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=2.500 mean_wait_max=2.500 mean_wait_mean=2.500
request_remote_free_policy_sample_summary=max_wait2 requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
request_remote_free_policy_sample_summary=max_queued256kib requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Host Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 24.544 us to 24.722 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 23.907 us to 24.150 us |
| `request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b` | 24.219 us to 24.410 us |

## Docker Results

Repeated request policy summaries:

```text
request_remote_free_policy_sample_summary=end_drain requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=16 max_pending_max=16 max_pending_mean=16.000 max_queued_bytes_min=524288 max_queued_bytes_max=524288 max_queued_bytes_mean=524288 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=2.500 mean_wait_max=2.500 mean_wait_mean=2.500
request_remote_free_policy_sample_summary=max_wait2 requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
request_remote_free_policy_sample_summary=max_queued256kib requests=16 bursts=4 burst_requests=4 capacity=16 batch_limit=8 samples=8 full_min=0 full_max=0 full_mean=0.000 policy_drains_min=2 policy_drains_max=2 policy_drains_mean=2.000 drain_rounds_min=2 drain_rounds_max=2 drain_rounds_mean=2.000 max_pending_min=8 max_pending_max=8 max_pending_mean=8.000 max_queued_bytes_min=262144 max_queued_bytes_max=262144 max_queued_bytes_mean=262144 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Docker Criterion timings:

| Benchmark | Timing |
| --- | ---: |
| `request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b` | 62.967 us to 66.696 us |
| `request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b` | 55.337 us to 65.416 us |
| `request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b` | 52.115 us to 62.880 us |

Criterion printed change percentages from existing benchmark history. They are
not used here because they compare against prior runs from a different local
state or environment.

## Interpretation

The postulate survived.

The queued-byte request policy matched max-wait-2 exactly for repeated counters
in both host and Docker runs:

- peak queued bytes: 262,144;
- max pending requests: 8;
- policy drains: 2;
- max wait: 2 bursts;
- mean wait: 1.500 bursts;
- `full_count=0`.

Compared with end-drain, the queued-byte policy reduced peak retained request
arena bytes by 50 percent and reduced max wait from 4 bursts to 2 bursts
without adding producer backpressure. The owner release path still closes real
request arenas through `RequestScratchPool::close_request` inside the drain
closure.

The host timing did not beat the current best request-scratch throughput result,
so the best-results note was not updated. The important result is that the
direct byte-budget policy now has evidence on request-affine arena returns,
KV handles, and mixed allocation traces.

## Next Step

The queued-byte remote-free policy now has enough cross-domain counter evidence
to justify a small runtime configuration note or helper for selecting retained
byte budgets from request concurrency, block size, arena size, and expected
release latency.
