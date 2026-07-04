# Postulate 0354: Sharded Chunk Publish Survives a Realistic KV Trace

Date: 2026-07-04

## Statement

The sharded chunk-publish advantage measured under commanded all-at-once
cycles (experiments 0352 and 0353) survives a realistic mixed
request-lifetime KV trace with continuous frees: staggered request
arrivals, prefill bursts, per-step decode block growth, mixed completion
lengths, and early cancellations, with worker threads freeing whole
request block sets as they complete while the owner keeps allocating.
Under this trace, per-worker chunk queues should beat one shared
per-handle queue by a measurable margin at four workers. If the advantage
disappears, the 0352/0353 wins were artifacts of the rendezvous-heavy
cycle shape and the design decision to shard must be revisited.

## Rationale

All remote-free benchmarks so far, including 0351 to 0353, drive
synchronized cycles: producers receive a command, enqueue a fixed block
set, and the owner drains to empty. Real serving frees continuously.
Request lifetimes overlap, frees arrive while the owner is allocating for
other requests, and the queue rarely drains to empty in lockstep. The
contention and per-item costs proven earlier might shrink under this
shape because frees are spread over time instead of bursting all at once.
Since 0353 recorded the design decision to shard with a chunk ABI, that
decision must be validated under the trace shape it is actually meant
for. This is also the substrate that jemalloc and mimalloc baselines will
run on next, so the trace itself is a deliverable.

## Experiment

Add a `remote_free_mixed_lifetime_trace` benchmark to `locus-alloc`:

- 64 requests, staggered 4 per step; each request allocates a 16-block
  prefill burst on arrival, then 1 block per step while active;
- request r runs 16, 32, or 48 decode steps by r % 3; every fourth
  request cancels early after 8 decode steps;
- on completion or cancellation the request's whole block vector is
  dispatched to one of 4 persistent worker threads round-robin, which
  frees it through the configured remote-free path immediately, with no
  per-cycle rendezvous;
- owner drains its queue(s) once per step and retries with drains if the
  pool is exhausted; blocks are real 4 KiB `KvBlockPool` allocations;
- modes: shared (one per-handle queue, capacity 1024, batch 64) versus
  sharded chunk (one Vec queue per worker, capacity 8);
- end-of-trace accounting proves every allocated block was freed back
  into the pool and every queue is balanced with pending 0.

One benchmark iteration is the full trace. Run the Criterion suite twice.

## Expected Result

Sharded chunk should beat shared per-handle at 4 workers under the trace,
though likely by less than the 1.9x seen in commanded cycles since frees
are temporally spread. A win of at least 10 percent keeps the sharded
chunk design decision. A null result falsifies the postulate and demotes
sharding to a burst-only optimization.
