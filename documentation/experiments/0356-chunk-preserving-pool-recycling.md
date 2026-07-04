# Experiment 0356: Chunk-Preserving Pool Recycling (Falsified)

Date: 2026-07-04

## Postulate

[Postulate 0356](../postulates/0356-chunk-preserving-pool-recycling.md)
claimed that preserving request chunk identity inside the pool
(`free_chunk` splicing whole request block groups into a recycled chunk
store, `allocate_chunk` serving prefill bursts from recycled chunk tails)
would cut the mixed-lifetime trace from 80 us to under 60 us.

## Change (implemented, measured, then reverted)

Temporarily added to `KvBlockPool`: a `recycled: Vec<Vec<usize>>` chunk
store with a running block total, `free_chunk` (per-handle validation
retained, list manipulation per chunk), `allocate_chunk` (recycled chunks
consumed most-recent first), a fallback so `allocate` drains recycled
chunks when the flat list empties, stats counting recycled blocks as
free, and four unit tests (chunk round-trip, stale-handle rejection,
exhaustion atomicity, single-allocate fallback; all passed alongside the
existing 191). Extended the mixed-lifetime trace benchmark with a
`sharded_chunk_pool` path using these APIs for prefill bursts and drained
chunk frees.

## Host Validation

```bash
cargo bench -p locus-alloc --bench remote_free_mixed_lifetime_trace -- --sample-size 20 --warm-up-time 1 --measurement-time 3   # run twice with the change
# control: same command on the committed tree (git stash), sharded_chunk only
```

All runs kept `allocated=10752 freed=10752` per sample.

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `..._sharded_chunk_w4_64req` (with pool change) | 92.926 us to 93.580 us | 87.444 us to 89.596 us |
| `..._sharded_chunk_pool_w4_64req` (chunk APIs) | 89.731 us to 91.137 us | 87.736 us to 88.692 us |
| `..._sharded_chunk_w4_64req` (committed tree control, same session) | 78.314 us to 80.422 us | not repeated |

An `#[inline]` variant on the pop helpers measured 89.7 to 93.4 us,
within the same band. Raw logs: scratchpad
`mixed_lifetime_chunk_pool_run1.log` and `mixed_lifetime_chunk_pool_run2.log`.

## Interpretation

Falsified on both counts, with a bonus negative:

1. Chunk-preserving pool ops gained nothing: sharded_chunk_pool was
   statistically indistinguishable from sharded_chunk in run 2 (88.2 vs
   88.3 us) and about 3 percent ahead in run 1, far from the predicted
   sub-60 us. Owner-side pool bookkeeping is not where the remaining
   time goes; drain polling and channel receive dominate.
2. Worse, merely carrying the recycled-chunk store regressed the
   untouched sharded_chunk path about 11 percent (79.4 us control vs
   88 to 93 us with the change), from the extra fallback branch and
   larger pool struct on the hot allocate path.
3. The locality argument for chunk preservation is also void on
   reflection: the flat free list is LIFO and a drained chunk is pushed
   in order, so per-handle frees already return burst-shaped, warm block
   groups. A separate chunk store duplicates what the stack gives free.

Design consequence: the pool change was reverted; `KvBlockPool` keeps
the single flat LIFO free list. Chunk identity pays at the transport
layer (0353 to 0355) and stops paying at the pool boundary. The
remaining 80 us is in the handoff machinery itself: bounded channel
send/receive, per-queue stats atomics, and owner polling across shards.

## Next Question

Can the handoff machinery be reduced to its theoretical minimum: one
lock-free chunk mailbox per worker (an atomic pointer stack; producers
CAS-push a request chunk node, the owner swap-takes the whole list),
with no capacity accounting, no stats atomics, and no bounded-channel
protocol on the free path? Does that cut the trace materially below the
80 us bounded-queue design?
