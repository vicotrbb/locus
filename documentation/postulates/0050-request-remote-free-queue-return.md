# Postulate 0050: Request Remote Free Queue Return

Date: 2026-07-02

## Statement

Request scratch arena return should be benchmarked through `RemoteFreeQueue` because request completion can happen away from the owning allocator worker.

## Rationale

The KV remote-free benchmark measures handle release for one allocator domain. Request-affine scratch arenas have a different lifecycle: the owner opens and allocates request-local arenas, while completion may arrive from scheduler or network paths. Returning request IDs through a remote-free queue lets the owner close and recycle arenas without transferring pool ownership across threads.

## Experiment

Add a benchmark that:

- owns a `RequestScratchPool` on the benchmark thread;
- opens 16 request arenas per iteration;
- performs 64 aligned 256-byte allocations per request;
- sends completed `RequestId`s to a persistent remote completion thread;
- enqueues request IDs into `RemoteFreeQueue`;
- drains the queue on the owner thread and calls `RequestScratchPool::close_request`.

## Expected Result

The benchmark should compile under all-target checks and produce a measurable request arena return baseline. It is expected to be slower than same-thread `request_scratch_pool_cycle_16x64x256b` because it includes remote completion and queue draining.
