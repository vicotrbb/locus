# Postulate 0038: Vec Uninit Capacity Benchmark Baseline

Date: 2026-07-02

## Statement

The allocator benchmark suite needs a default global allocator baseline that avoids byte initialization costs and complements the existing `Vec<u8>` baseline.

## Rationale

The current `Vec<u8>` baselines measure ordinary Rust allocation plus zero initialization. Arena paths usually hand back uninitialized reusable memory, so a direct comparison against zero-filled vectors can overstate allocator advantages for small scratch and KV block cycles.

`Vec::<MaybeUninit<u8>>::with_capacity` exercises the default global allocator without manual unsafe code and without initializing each byte. This keeps the benchmark inside the repository safety policy, stays compatible with the Rust 1.80 MSRV, and makes the baseline more comparable to arena allocation.

## Experiment

Add benchmark cases for:

- 64 allocations with 256 bytes of uninitialized vector capacity;
- 256 allocations with 4096 bytes of uninitialized vector capacity.

Compare those cases with the existing scratch arena, KV block pool, and `Vec<u8>` allocation cycle benchmarks.

## Expected Result

The new baselines should compile under all-target checks. They should run slower than reusable arena allocation but faster than the zero-filled `Vec<u8>` baselines when initialization cost is significant.
