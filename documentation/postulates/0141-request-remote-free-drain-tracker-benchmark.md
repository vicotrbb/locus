# Postulate 0141: Request Remote-Free Drain Tracker Benchmark

Date: 2026-07-03

## Claim

`RemoteFreeDrainTracker` should preserve remote-free policy signals for request-scratch arena returns, not only for `Vec` buffers and KV block handles.

## Rationale

Experiment 0148 validated tracker-backed policy accounting on a KV block handle path. Request-affine scratch arenas have a different lifecycle: the owner opens request arenas, allocates request-local scratch memory, and later receives request IDs from remote completion paths to close and recycle arenas.

The policy should still expose the same retained-memory and release-latency tradeoff:

- large end-drain queues reduce producer backpressure but retain more arenas longer;
- max-wait policy should reduce pending request count, retained arena bytes, and logical wait without increasing queue-full backpressure.

## Experiment

Add a focused `request_remote_free_policy` benchmark target.

Use:

- `RequestScratchPool` with 16 requests;
- 32 KiB arena capacity per request;
- 64 allocations of 256 bytes per request;
- `RemoteFreeQueue<RequestId>` with capacity 16 and batch 8;
- a persistent remote completion thread;
- four bursts of four request IDs;
- `RemoteFreeDrainTracker` with 32 KiB recorded per pending request;
- end-drain and max-wait-2 policies.

## Falsification

The postulate is weakened if max-wait-2 fails to reduce pending requests, queued arena bytes, or wait bursts versus end-drain, or if tracker observations drift from queue pending counts.

## Expected Value

If the postulate survives, `RemoteFreeDrainTracker` will have evidence across byte buffers, KV handles, and request-affine arena returns.
