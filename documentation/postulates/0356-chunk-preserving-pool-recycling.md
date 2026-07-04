# Postulate 0356: Chunk-Preserving Pool Recycling Cuts Owner-Side Cost

Date: 2026-07-04

## Statement

Preserving request chunk identity all the way into the pool removes most
of the owner-side per-handle bookkeeping that remains in the 80 us
mixed-lifetime trace. Concretely: adding `free_chunk` (validate then
splice a whole request's blocks into the pool as one intact recycled
chunk) and `allocate_chunk` (serve a prefill burst from a recycled chunk
tail instead of popping the free list once per block) should cut the
sharded chunk trace below 60 us. This is a design idea general-purpose
allocators do not offer: they dissolve freed groups into size-class free
lists, while KV serving frees blocks in request-shaped groups and
reallocates them in burst-shaped groups of similar size, so group
identity is worth preserving end to end.

## Rationale

After 0354 and 0355 the transport is chunked (one queue item per request)
but the pool boundary still degrades chunks into 2688 per-handle `free`
calls (validate, generation bump, free-list push) and 2688 per-handle
`allocate` calls per trace. If those loops are a large share of the 80 us,
chunk-preserving pool APIs attack it directly: validation stays per
handle (safety is non-negotiable) but list manipulation becomes per
chunk, and prefill bursts reuse a recently freed request's blocks
together, which also keeps their metadata cache-warm. The result decides
whether the locus allocator ABI should be chunk-first at every layer,
which is its clearest differentiation from jemalloc and mimalloc.

## Experiment

1. Add to `KvBlockPool`: a recycled chunk store (`Vec<Vec<usize>>` plus a
   running block total), `free_chunk`, `allocate_chunk`, fallback so
   `allocate` drains recycled chunks when the flat free list is empty,
   and stats counting recycled blocks as free. Unit tests cover chunk
   round-trips, stale-handle rejection inside chunks, exhaustion, and
   mixed chunk plus flat usage.
2. Extend the mixed-lifetime trace benchmark with a third path,
   `sharded_chunk_pool`: identical transport to sharded_chunk, but the
   owner frees drained request vectors via `free_chunk` and serves
   prefill bursts via `allocate_chunk(16)`; decode still allocates single
   blocks. End-of-trace accounting proves allocated == freed == 2688 and
   pool stats balance.
3. Run the full suite twice and compare sharded_chunk against
   sharded_chunk_pool on the same runs.

## Expected Result

sharded_chunk_pool should land clearly below sharded_chunk (80 us), with
under 60 us counted as a clean survival. A result within noise of 80 us
falsifies the postulate and means owner-side cost is dominated by drain
polling and channel receive, not pool bookkeeping.
