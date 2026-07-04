# Experiment 0353: Chunk Publish vs Per-Handle Sharded Remote Free

Date: 2026-07-04

## Postulate

[Postulate 0353](../postulates/0353-remote-free-chunk-publish.md) claimed
the 12.8 us p4 floor left after sharding is dominated by per-handle enqueue
cost, so publishing each producer's chunk as one queue item should cut p4
below 10 us.

## Change

Added `crates/locus-alloc/benches/remote_free_chunk_publish.rs` with four
benchmarks at p2 and p4, same cycle shape as 0352 (persistent producers,
256 real 4 KiB blocks from a `KvBlockPool`, concurrent enqueue and drain):

- `kv_remote_free_chunkcmp_per_handle_p{2,4}_batch64_256x4k`: per-producer
  capacity-(256/p) `RemoteFreeQueue<KvBlockHandle>`, 256/p sends each;
- `kv_remote_free_chunkcmp_chunk_p{2,4}_256x4k`: per-producer capacity-2
  `RemoteFreeQueue<Vec<KvBlockHandle>>`, one send per producer per cycle,
  owner drains vectors and frees every handle.

Pre-benchmark samples assert per-queue submitted == drained, pending == 0,
and total items (2048 handles per-handle, 8 * p chunks) match exactly.

## Host Validation

```bash
cargo bench -p locus-alloc --bench remote_free_chunk_publish -- --sample-size 30 --warm-up-time 1 --measurement-time 2   # run twice
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

All counters balanced in both runs, zero full, zero disconnected.

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `kv_remote_free_chunkcmp_per_handle_p2_batch64_256x4k` | 9.2295 us to 9.3669 us | 9.1346 us to 9.2451 us |
| `kv_remote_free_chunkcmp_chunk_p2_256x4k` | 7.2801 us to 7.3773 us | 7.3594 us to 7.5666 us |
| `kv_remote_free_chunkcmp_per_handle_p4_batch64_256x4k` | 12.725 us to 12.976 us | 13.217 us to 13.580 us |
| `kv_remote_free_chunkcmp_chunk_p4_256x4k` | 10.644 us to 10.978 us | 10.689 us to 10.933 us |

Raw logs: scratchpad `remote_free_chunk_publish_run1.log` and
`remote_free_chunk_publish_run2.log`. The per-handle p4 case reproduces the
0352 sharded result (12.7 to 13.6 us), a good cross-benchmark consistency
check.

## Interpretation

The postulate survives in direction but fails its magnitude claim again,
and the two failures together now pin down the cost model:

1. Chunk publish wins consistently: about 1.9 us at p2 (9.2 to 7.3) and
   about 2.2 us at p4 (13.0 to 10.8), reproduced in both runs. But p4 does
   not go below 10 us.
2. The saving is roughly constant at about 2 us per cycle regardless of
   producer count, which is the signature of a per-item term: 256 sends
   collapse to a handful, saving about 8 ns per handle send, independent of
   how many threads produced them.
3. What remains is a per-active-thread coordination cost. Across 0351,
   0352, and 0353 the best times are 5.3 us at p1, 7.3 us at p2 sharded
   chunk, 10.8 us at p4 sharded chunk: a clean floor of roughly 1.8 us per
   additional producer thread that survives removing contention (0352) and
   removing per-item sends (0353). That is thread wake-up and rendezvous,
   not channel mechanics.

Design consequence: the remote-free path should combine both levers, one
queue per producer and a batch-publish ABI (accept `Vec` or linked chunks
of handles, splice on drain). Together they take p4 from 21 us to 10.8 us,
a 1.94x reproduced improvement. Beyond that, this fan-out shape is bounded
by scheduler wake-up latency on this host, so further gains must come from
avoiding synchronous rendezvous entirely, that is, producers that free
continuously without a per-cycle command hand-off, which is also the more
realistic serving shape.

## Next Question

Move to priority 2 of the research direction: under a realistic mixed
request-lifetime KV trace (prefill burst, steady-state decode churn, early
cancellation) where producers free continuously rather than in commanded
cycles, does the sharded chunk-publish design still beat the shared queue,
and how do jemalloc and mimalloc baselines compare on the same trace?
