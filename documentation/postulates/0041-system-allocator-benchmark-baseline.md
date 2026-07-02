# Postulate 0041: System Allocator Benchmark Baseline

Date: 2026-07-02

## Statement

An explicit `std::alloc::System` benchmark binary should document the libc or platform system allocator baseline without relying on raw `malloc` and `free` calls.

## Rationale

The default Rust allocator is target-dependent. On common Unix targets it is system allocation, but the benchmark suite should make allocator identity explicit when comparing against mimalloc and jemalloc.

Using `std::alloc::System` as the global allocator in a dedicated Criterion target keeps the benchmark safe, avoids manual FFI allocation code, and gives a named system allocator baseline for the matrix.

## Experiment

Add a `scratch_arena_system` benchmark target that measures:

- system-backed 64 by 256-byte zero-filled `Vec<u8>` allocation;
- system-backed 64 by 256-byte uninitialized vector capacity allocation;
- system-backed 256 by 4096-byte zero-filled `Vec<u8>` allocation;
- system-backed 256 by 4096-byte uninitialized vector capacity allocation.

## Expected Result

The new benchmark target should compile under all-target checks and produce focused system allocator timings. On this platform, it is expected to be close to the default allocator samples.
