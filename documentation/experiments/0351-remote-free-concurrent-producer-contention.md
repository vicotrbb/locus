# Experiment 0351: Remote-Free Concurrent Producer Contention

Date: 2026-07-04

## Postulate

[Postulate 0343](../postulates/0343-remote-free-concurrent-producer-contention.md) claimed that the large-batch advantage from the single-threaded KV remote-free sweep in experiment 0059 would not carry over to real concurrent producers, and that producer count would matter at least as much as batch limit.

## Change

Added `crates/locus-alloc/benches/remote_free_concurrent.rs` with nine benchmarks:

- `kv_remote_free_concurrent_p{1,2,4}_batch{8,64,256}_256x4k`

Each case pre-spawns persistent producer threads that share one `RemoteFreeSink<KvBlockHandle>`. Every iteration allocates 256 real 4 KiB blocks from a `KvBlockPool`, splits them evenly across the producers, and the producers enqueue concurrently while the owner thread drains with the configured batch limit and frees every handle back into the pool. The pre-benchmark sample runs eight full cycles and asserts `submitted == drained` and `pending == 0`.

This is the first remote-free benchmark where enqueue and drain genuinely overlap across threads. All earlier sweeps enqueued everything first and drained afterward.

## Host Validation

Commands:

```bash
cargo bench -p locus-alloc --bench remote_free_concurrent -- --sample-size 30 --warm-up-time 1 --measurement-time 2   # run twice
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

Pre-benchmark counters for every case: `submitted=2048 drained=2048 pending=0 full=0 disconnected=0`. No blocking enqueue ever observed a dropped owner, and no items were lost.

Host Criterion timings, two independent runs:

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `kv_remote_free_concurrent_p1_batch8_256x4k` | 5.7772 us to 5.9446 us | 5.2101 us to 5.3343 us |
| `kv_remote_free_concurrent_p1_batch64_256x4k` | 6.2544 us to 6.8696 us | 6.0058 us to 8.7463 us |
| `kv_remote_free_concurrent_p1_batch256_256x4k` | 7.3012 us to 18.869 us | 5.2502 us to 5.3943 us |
| `kv_remote_free_concurrent_p2_batch8_256x4k` | 15.203 us to 44.720 us | 9.4561 us to 9.6382 us |
| `kv_remote_free_concurrent_p2_batch64_256x4k` | 10.389 us to 11.179 us | 10.942 us to 18.663 us |
| `kv_remote_free_concurrent_p2_batch256_256x4k` | 11.122 us to 19.641 us | 9.5403 us to 9.8058 us |
| `kv_remote_free_concurrent_p4_batch8_256x4k` | 23.298 us to 52.889 us | 23.244 us to 23.772 us |
| `kv_remote_free_concurrent_p4_batch64_256x4k` | 23.617 us to 24.791 us | 20.943 us to 21.518 us |
| `kv_remote_free_concurrent_p4_batch256_256x4k` | 21.017 us to 21.405 us | 20.931 us to 21.273 us |

Raw logs: scratchpad `remote_free_concurrent_run1.log` and `remote_free_concurrent_run2.log` (Criterion reports under `target/criterion/kv_remote_free_concurrent_*`).

## Interpretation

The postulate survives in its main claim and needs refinement in one detail:

1. The single-threaded batch ranking does not carry over. In experiment 0059, batch 256 was roughly 6.6x faster than batch 8 (5.55 us vs 36.9 us). With concurrent drain, batch 8 and batch 256 are statistically indistinguishable at every producer count in the cleaner run 2. The 0059 advantage came from the all-at-once shape, where small batches forced many empty polling rounds; when drain overlaps production, small batches drain continuously and lose nothing.
2. Producer count dominates. Median cycle time scales roughly 5.3 us at p1, 9.6 us at p2, and 21 us at p4 for the same 256 blocks, so send-side contention on the shared bounded channel costs about 4x when going from one to four producers. Batch limit shifts nothing comparable.
3. Cross-run variance is much higher than in single-threaded sweeps. Several cases show 2x to 3x wide confidence intervals in one run and tight ones in the other, which is expected scheduler noise but means single runs of concurrent benchmarks must not be trusted for ranking.

Policy consequence: batch-limit tuning derived from single-threaded sweeps is not a valid lever for concurrent serving shapes. Reducing producer-side contention, for example by sharding remote-free queues per node or per producer group, is the lever this data points at.

## Next Question

Does sharding remote frees across multiple owner queues, one per producer, remove the producer-count penalty, or does the drain-side aggregation cost eat the win? Measure p4 with four capacity-64 queues against p4 with one shared capacity-256 queue at the same total block count.
