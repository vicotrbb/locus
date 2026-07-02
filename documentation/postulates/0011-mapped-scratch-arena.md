# Postulate 0011: Mapped Scratch Arena

Date: 2026-07-02

## Statement

An mmap-backed scratch arena should preserve the safe arena API while moving the allocation substrate closer to Linux page-placement validation.

## Rationale

The existing scratch arena and request pools are Vec-backed. That is useful for API and lifecycle validation, but Linux memory policy and page residency checks operate on mapped virtual address ranges. A mapped scratch arena is the smallest allocator change that consumes the new `locus-sys` system boundary.

## Experiment

Add `MappedScratchArena` that:

- owns a `MappedRegion`;
- exposes safe alignment-aware scratch allocation;
- records the same `ScratchArenaStats` as the Vec-backed arena;
- has focused tests for alignment, reset accounting, and out-of-memory behavior;
- benchmarks reset-cycle allocation against the existing scratch and `Vec<u8>` baselines.

## Expected Result

The mapped arena should pass the workspace gates and provide a benchmarked bridge between safe allocator APIs and future Linux NUMA memory-policy work.
