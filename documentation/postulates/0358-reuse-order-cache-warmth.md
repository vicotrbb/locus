# Postulate 0358: Reuse Order Matters Once Blocks Are Actually Touched

Date: 2026-07-04

## Statement

All trace experiments so far never write into block memory, so they
cannot see cache effects of reuse order. Once every allocated block is
actually written (as a serving engine writes KV entries), LIFO reuse
(most recently freed block reused first, the current flat free list)
should beat FIFO reuse (oldest freed block reused first) measurably,
because LIFO keeps the reused block's lines resident in cache while FIFO
maximizes reuse distance. The margin quantifies the cache-warmth value
of recency, which directly bounds how much reuse reordering a future
NUMA placement layer may impose: if the margin is large, node-preferring
reuse must still respect recency; if it is small, placement policy can
reorder reuse freely.

## Rationale

The pool free list is a Vec used LIFO, which is standard practice, but
its value has never been measured here, and the mixed-lifetime trace
results (0354 to 0357) are pure handoff plus bookkeeping. On this host
(Apple M-series, large shared L2 and SLC), 4 KiB blocks recycled quickly
may stay warm across free and reallocate, so writes into a LIFO-reused
block avoid DRAM traffic that FIFO-reused blocks pay. A NUMA-aware
recycler will want to steer chunks to node-local consumers, which
perturbs recency order; this experiment prices that perturbation.

## Experiment

1. Add a reuse-order policy to `KvBlockPool`: the free list becomes a
   `VecDeque<usize>`; `KvReuseOrder::Lifo` pops the back (current
   behavior), `KvReuseOrder::Fifo` pops the front. Default constructor
   keeps LIFO; a new constructor selects the order. Unit tests pin both
   orders. Rerun the mixed-lifetime sharded_mailbox benchmark to confirm
   the VecDeque swap does not regress the untouched path.
2. Add a `kv_reuse_order_locality` benchmark: single-owner churn loop
   with a working set large enough to defeat the caches under FIFO,
   64 MiB pool (16384 blocks of 4 KiB). Steady state: allocate a
   16-block chunk, write every byte of each block, free the oldest
   live chunk; 256 live chunks (16 MiB live set); measure cycles of 64
   allocate-touch-free steps. Cases: lifo_touch, fifo_touch, and both
   orders without touches as controls.
3. Two Criterion runs; controls must show no order effect for the
   result to be attributed to cache warmth.

## Expected Result

lifo_touch beats fifo_touch by at least 10 percent while untouched
controls are within noise of each other. If touched cases are also
within noise, recency warmth is worthless at KV block scale on this
memory system and NUMA placement gets free rein over reuse order; that
negative would be as design-relevant as the positive.
