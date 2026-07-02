# Postulate 0046: Remote Free Queue Primitive

Date: 2026-07-02

## Statement

Locus needs a small owner-drained remote free queue before designing allocator-specific remote-free batching.

## Rationale

The handoff benchmarks show that cross-thread allocation and release behavior is measurable and allocator-dependent. A Locus-owned primitive should model the runtime pattern directly: remote producers enqueue items, while the owning worker drains and releases items in bounded batches.

Keeping the first primitive generic and safe lets later experiments plug in KV block handles, request arenas, or other owned resources without committing to one pool design too early.

## Experiment

Add a `RemoteFreeQueue<T>` with:

- cloneable `RemoteFreeSink<T>` handles for remote producers;
- bounded queue capacity;
- configured drain batch limit;
- owner-side `drain_batch` with a release closure;
- submitted and drained accounting;
- tests for batch draining, invalid configuration, and closed-owner enqueue errors.

## Expected Result

The primitive should pass workspace tests and clippy without unsafe code. It should provide the API surface needed for a benchmark against persistent-worker allocator baselines.
