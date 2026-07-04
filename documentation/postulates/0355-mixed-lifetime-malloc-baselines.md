# Postulate 0355: Locus Sharded Chunk Beats General-Purpose Malloc on the KV Trace

Date: 2026-07-04

## Statement

On the mixed-lifetime KV trace from experiment 0354, the locus block pool
with sharded chunk remote free (80 us per trace) beats jemalloc, mimalloc,
and the macOS system allocator running the same trace with raw 4 KiB heap
allocations freed natively on worker threads. Expected ordering: locus
fastest, then mimalloc, then jemalloc, then system. If any general-purpose
allocator matches or beats the locus path, the pool plus remote-free design
is not paying for itself and the project direction must be reassessed.

## Rationale

Experiments 0351 to 0354 optimized the locus remote-free path against
itself. The design only matters if it beats what a serving engine gets for
free by calling malloc from any thread. General-purpose allocators handle
cross-thread frees with highly tuned thread caches and deferred reclaim
(mimalloc's free-list sharding, jemalloc's tcache plus arena model), which
is exactly the competition for a KV block pool. A 4 KiB fixed-size churn
with cross-thread frees is close to the best case for thread-caching
mallocs, so this is a fair and hard baseline, and the result decides
whether locus should keep investing in the pooled remote-free design or
pivot.

## Experiment

Add three bench binaries sharing one trace module
(`benches/mixed_lifetime_malloc/trace.rs`):

- `mixed_lifetime_jemalloc` with `tikv_jemallocator::Jemalloc` global;
- `mixed_lifetime_mimalloc` with `mimalloc::MiMalloc` global;
- `mixed_lifetime_system` with the default macOS allocator.

The trace is byte-identical in shape to 0354: 64 requests, 4 arrivals per
step, 16-block prefill, decode growth of one 4 KiB block per step, decode
lengths 16/32/48, every fourth request cancels after 8 steps, 2688 blocks
per trace. Blocks are `Vec<u8>` created with `Vec::with_capacity(4096)`
plus a first-byte write; completed requests dispatch their block vectors
to 4 persistent workers that drop them on the worker thread, the native
cross-thread free path. An `AtomicUsize` counts worker-side frees; each
iteration waits until freed == allocated == 2688 so no free is deferred
past the measurement or leaked. Two Criterion runs per the concurrency
rule, compared against the 0354 locus sharded chunk numbers on this host.

## Expected Result

Locus sharded chunk (80 us) should hold a clear lead; mimalloc is
expected closest, likely 1.2x to 2x slower, with jemalloc and system
behind. Falsified if any baseline reaches 80 us or better.
