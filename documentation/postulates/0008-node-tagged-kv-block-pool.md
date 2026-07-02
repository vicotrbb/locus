# Postulate 0008: Node-Tagged KV Block Pool

Date: 2026-07-02

## Statement

A safe fixed-size KV block pool is the smallest useful foundation for studying KV-cache allocation behavior before implementing full paged KV tables.

## Rationale

The research notes identify KV cache growth and reuse as a primary allocator pressure point in LLM serving. A fixed-size block pool lets Locus model block allocation, reuse, stale-handle detection, high-water accounting, and node tagging without introducing GPU kernels or Linux memory-policy syscalls.

## Experiment

Add a `KvBlockPool` that:

- owns a fixed number of fixed-size blocks;
- allocates and frees opaque block handles;
- detects stale or double-freed handles with generations;
- tracks high-water, allocation, and free counts;
- benchmarks block allocate/free against `Vec<u8>` block allocation.

## Expected Result

The pool should pass correctness tests and provide a repeatable baseline for KV block reuse. It should not claim page placement or GPU compatibility yet.
