# Experiment 0352: Sharded Remote-Free Producer Queues

Date: 2026-07-04

## Postulate

[Postulate 0352](../postulates/0352-remote-free-sharded-producer-queues.md)
claimed the p4 producer-count penalty from experiment 0351 is send-side
contention on the shared bounded queue, so one queue per producer (owner
draining round-robin) should recover most of it, landing p4 sharded near the
p1 shared baseline.

## Change

Added `crates/locus-alloc/benches/remote_free_sharded.rs` with four
benchmarks at fixed batch limit 64:

- `kv_remote_free_shared_p{2,4}_batch64_256x4k`: one capacity-256 queue,
  all producers share one sink;
- `kv_remote_free_sharded_p{2,4}_batch64_256x4k`: one capacity-(256/p)
  queue per producer, owner drains all shards round-robin.

Same shape as 0351: persistent producer threads, 256 real 4 KiB blocks
allocated from a `KvBlockPool` per cycle, enqueue and drain genuinely
overlap, every handle freed back into the pool. Pre-benchmark sample runs
eight cycles and asserts per-queue submitted == drained, pending == 0, and
total submitted == 2048.

## Host Validation

```bash
cargo bench -p locus-alloc --bench remote_free_sharded -- --sample-size 30 --warm-up-time 1 --measurement-time 2   # run twice
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

Counters in both runs: every queue balanced, zero full, zero disconnected,
zero pending; sharded shards each carried exactly 2048/p items.

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `kv_remote_free_shared_p2_batch64_256x4k` | 9.8325 us to 10.016 us | 9.7161 us to 9.8894 us |
| `kv_remote_free_sharded_p2_batch64_256x4k` | 9.1949 us to 9.3900 us | 9.1896 us to 9.3012 us |
| `kv_remote_free_shared_p4_batch64_256x4k` | 20.698 us to 21.213 us | 20.448 us to 20.883 us |
| `kv_remote_free_sharded_p4_batch64_256x4k` | 12.712 us to 12.899 us | 12.569 us to 13.059 us |

Raw logs: scratchpad `remote_free_sharded_run1.log` and
`remote_free_sharded_run2.log`. Unlike 0351, confidence intervals were tight
in both runs; the two runs agree within 2 percent on every case.

## Interpretation

The postulate survives in direction but fails its magnitude claim, and the
refinement is the interesting result:

1. Sharding wins clearly at p4: 12.8 us versus 20.9 us, a 1.63x improvement,
   reproduced in both runs. Send-side contention on the shared queue is real
   and removable.
2. But sharded p4 lands nowhere near the p1 baseline (5.3 us from 0351) and
   does not even reach shared p2 (9.9 us). Roughly half of the p4 penalty is
   not producer-producer channel contention.
3. The p2 comparison is the diagnostic: sharding at p2 gains only about 6
   percent (9.2 vs 9.8 us). With only two producers, channel contention is
   nearly free; the cost that remains is per-thread fan-out overhead
   (command wake-up, scheduler placement, and each producer's own
   sequential enqueue loop) plus drain-side polling across shards.
4. Implied cost model: cycle time is roughly a fixed per-active-thread
   coordination cost plus a contention term that grows with producers on a
   shared queue. Sharding deletes the contention term (about 8 us at p4)
   but cannot touch the coordination floor (about 12 to 13 us at p4).

Design consequence: per-producer (or per-node) sharded remote-free queues
are worth having, they buy 1.6x at four producers with no loss anywhere and
make enqueue spsc-shaped. But sharding alone will not make remote frees
scale flat with producer count; the remaining floor is thread coordination,
which points at producer-side local batching (accumulate frees locally,
publish in chunks) as the next lever, since it amortizes the wake-up and
enqueue path rather than the channel.

## Next Question

Does producer-side local batching (each producer accumulates its 64 handles
and publishes them as one chunk per cycle, for example via a Vec swap or a
chunk channel) cut the remaining p4 coordination floor below sharding alone?
Measure p4 sharded per-handle enqueue against p4 sharded chunk publish at
the same 256-block cycle.
