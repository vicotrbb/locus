# Postulate 0039: Mimalloc Benchmark Baseline

Date: 2026-07-02

## Statement

An isolated mimalloc benchmark binary should provide the first industry allocator baseline without changing the default allocator used by the existing benchmark suite.

## Rationale

The current benchmark suite compares Locus pools against default Rust allocation through `Vec<u8>` and `Vec<MaybeUninit<u8>>::with_capacity`. The research notes identify mimalloc as a relevant low-contention allocator baseline, but swapping the global allocator for the existing benchmark binary would make default Rust results harder to interpret.

A separate Criterion bench binary with `mimalloc` installed as its `#[global_allocator]` lets Locus collect mimalloc-backed `Vec` baselines while preserving the existing default-allocator benchmark file.

## Experiment

Add a `scratch_arena_mimalloc` benchmark target that measures:

- mimalloc-backed 64 by 256-byte zero-filled `Vec<u8>` allocation;
- mimalloc-backed 64 by 256-byte uninitialized vector capacity allocation;
- mimalloc-backed 256 by 4096-byte zero-filled `Vec<u8>` allocation;
- mimalloc-backed 256 by 4096-byte uninitialized vector capacity allocation.

## Expected Result

The new benchmark target should compile under all-target checks and produce focused mimalloc baseline timings. It should not affect the existing `scratch_arena` benchmark binary.
