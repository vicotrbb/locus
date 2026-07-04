# Postulate 0343: Remote-Free Concurrent Producer Contention

Date: 2026-07-04

## Statement

The large-batch advantage measured for single-threaded KV remote-free release does not automatically carry over to real concurrent producers. Under multiple producer threads enqueueing into one bounded remote-free queue while the owner drains concurrently, contention on the shared channel and drain cadence should change the batch-limit sweet spot, and producer count should matter at least as much as batch limit.

## Rationale

Experiment 0059 showed batch 256 was the fastest KV remote-free sweep point, but every existing remote-free benchmark enqueues from at most one worker thread, and most run the enqueue and drain phases on the same thread in sequence. Real inference serving frees KV blocks from several worker threads at once while the owning shard drains. That shape introduces:

- send-side contention on the bounded channel among producers;
- concurrent drain progress that empties the queue while producers refill it;
- scheduling noise that a single-threaded sweep never observes.

If the single-threaded batch ranking survives real contention, batch limit is a robust tuning lever. If it does not, allocator policy work should tune against concurrent traces, not single-threaded sweeps.

## Experiment

Add a `remote_free_concurrent` benchmark to `locus-alloc` that:

- pre-allocates 256 real 4 KiB KV block handles from a `KvBlockPool`;
- spawns persistent producer threads (1, 2, and 4) that each enqueue their share of handles into one shared `RemoteFreeSink` on command;
- has the owner thread concurrently drain with batch limits 8, 64, and 256, freeing every handle back into the pool;
- re-allocates the handles each iteration so the pool free-list cycle stays real;
- records queue stats before benchmarking to prove no items were lost and to expose full counts.

## Expected Result

Concurrent producers should slow total handoff versus the single-threaded all-at-once sweep. The batch-limit ranking may compress or invert: with concurrent drain, small batches drain overlapping with production, so batch 8 should lose less than the single-threaded sweep suggested, and batch 256 should gain less. Producer count is expected to shift timings more than batch limit at the same total block count.
