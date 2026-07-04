# The Chunk-Granular Mailbox Design (Experiments 0354 to 0358)

Date: 2026-07-04

Five experiments after the concurrency lessons note, the remote-free
thread has converged on a design an LLM serving engine could adopt
today, each element evidence-backed and two elements contradicting
standard allocator practice.

## The design

KV block memory is owned by one pool per (future) NUMA node. Worker
threads never free blocks individually: a finished or cancelled request
returns all of its blocks as one chunk, pushed into a per-worker
lock-free mailbox (one CAS per request). The owner sweeps mailboxes with
one atomic swap each, frees the chunk into a flat LIFO free list, and
serves new requests newest-blocks-first.

## Evidence

1. Chunk ABI: on a realistic mixed-lifetime trace (staggered arrivals,
   prefill bursts, decode growth, early cancels, 2688 real 4 KiB blocks),
   sharded chunk transport beats a shared per-handle queue 3.5x
   (80 vs 282 us, experiment 0354).
2. Against general-purpose allocators on the identical trace: mimalloc
   198 us, jemalloc 230 us, macOS system 226 to 278 us. The mailbox
   variant of locus runs 68 us, about 2.9x faster than mimalloc, the
   strongest baseline (0355, 0357, 0358 regression check).
3. A KV pool with a naive shared free queue (282 us) is slower than just
   calling jemalloc (0355). Pooling alone is worthless; the chunk-shaped
   handoff is where the win lives.
4. Mailboxes beat bounded queues by 6 to 8 percent with far tighter
   tails, and delete every tuning parameter: no capacity, no batch
   limit, no queued-byte budget, no retune telemetry (0357). The entire
   knob-tuning surface locus built earlier (0059, 0142, 0143, 0146) is
   unnecessary on this path.
5. Chunk identity must stop at the pool boundary: a chunk-preserving
   recycled store inside the pool gained nothing and regressed the hot
   path 11 percent; the flat LIFO list already returns burst-shaped
   groups (0356, falsified and reverted).
6. Reuse recency is worth real bandwidth: once blocks are actually
   written, LIFO reuse beats FIFO by 11 to 18 percent, about 18 ns per
   4 KiB block (0358). A NUMA placement layer may steer chunks between
   node pools but must reuse newest-first within a node.

## Why this is not just mimalloc again

mimalloc and snmalloc batch remote frees by allocator-internal structure
(per page or per segment free lists) because they cannot know which
allocations die together. A KV-cache runtime knows exactly which blocks
die together: a request's. Batching ownership transfer by caller-defined
request lifetime is the piece general-purpose allocators cannot have,
and it is what removes both the contention (0351/0352) and the tuning
surface (0357) at once.

## What a serving engine can use today

Free KV blocks per request, not per block; one lock-free mailbox per
worker thread; flat LIFO recycling per node; no queue capacity tuning.
Expected effect at four workers on Apple Silicon class hardware: about
4x lower free-path overhead than a shared-queue pool and about 3x lower
than routing KV churn through the best general-purpose malloc.

## Open thread

NUMA validation (priority 4): the design is single-node-proven only.
Next step is a cfg-gated Linux placement experiment (first-touch node
binding, verified via get_mempolicy) runnable unchanged in OrbStack
Docker here and on a real multi-node host.
