# Postulate 0040: Jemalloc Benchmark Baseline

Date: 2026-07-02

## Statement

A separate jemalloc benchmark binary should provide another industry allocator baseline without changing the default or mimalloc benchmark binaries.

## Rationale

The research notes identify jemalloc as a mature host allocator baseline because of its arena, size-class, and fragmentation behavior. Locus needs measurements against jemalloc, but the benchmark should remain isolated so allocator-specific results are not mixed in one binary.

Using a dedicated Criterion target with jemalloc installed as the global allocator mirrors the mimalloc setup and keeps the benchmark suite explicit.

## Experiment

Add a `scratch_arena_jemalloc` benchmark target that measures:

- jemalloc-backed 64 by 256-byte zero-filled `Vec<u8>` allocation;
- jemalloc-backed 64 by 256-byte uninitialized vector capacity allocation;
- jemalloc-backed 256 by 4096-byte zero-filled `Vec<u8>` allocation;
- jemalloc-backed 256 by 4096-byte uninitialized vector capacity allocation.

## Expected Result

The new benchmark target should compile under all-target checks and produce focused jemalloc baseline timings. It should not affect the existing default or mimalloc benchmark binaries.
