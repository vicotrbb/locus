# Postulate 0043: Allocator Specific Handoff Benchmarks

Date: 2026-07-02

## Statement

The producer and consumer handoff benchmark should run under the isolated mimalloc, jemalloc, and explicit system allocator benchmark binaries.

## Rationale

The first handoff baseline measures cross-thread allocation and drop behavior under the default benchmark binary. Remote-free research needs allocator-specific evidence because general-purpose allocators differ in how they handle allocation on one thread and release on another.

Keeping the handoff cases in separate benchmark binaries preserves explicit global allocator identity and avoids mixing default, mimalloc, jemalloc, and system allocator results.

## Experiment

Add matching 256 by 4096-byte producer and consumer handoff benchmarks to:

- `scratch_arena_mimalloc`;
- `scratch_arena_jemalloc`;
- `scratch_arena_system`.

Each benchmark should allocate vectors on a producer thread, send them through a bounded channel, and drop them on a consumer thread.

## Expected Result

The new benchmark cases should compile under all-target checks and produce allocator-specific handoff timings. The results are expected to be slower than single-thread allocation cases because they include thread spawn, channel handoff, allocation, and cross-thread drop costs.
