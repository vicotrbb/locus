# Postulate 0126: Scratch Arena Module

Date: 2026-07-03

## Statement

The Vec-backed `ScratchArena` should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

`ScratchArena` is the first Locus domain allocator experiment and the base primitive used by request-affine scratch managers. It owns alignment checks, bump allocation, reset accounting, capacity accounting, and allocation failures. Keeping it in `src/lib.rs` mixes the base allocator fast path with mapped arenas, pinned pools, KV-cache allocation, remote-free infrastructure, and parser code.

Moving the base scratch arena into `scratch_arena.rs` should reduce root file size, isolate the reusable bump allocator logic, and keep the public API source compatible through root re-exports.

## Experiment

Extract the base scratch arena subsystem into `crates/locus-alloc/src/scratch_arena.rs`.

The module should own:

- `ScratchArena`;
- `ScratchArenaStats`;
- `ScratchAllocError`;
- private alignment rounding for the Vec-backed arena;
- focused tests for alignment, reset accounting, out-of-memory behavior, and unsupported alignment.

The module can depend on the root alignment limit constant. Mapped scratch arena code should remain in the root for this experiment.

## Real Workload Gate

The extraction must still pass the existing small-allocation and request-affine benchmark paths:

- `scratch_arena_reset_cycle_64x256b`;
- `vec_allocation_cycle_64x256b`;
- `vec_uninit_capacity_allocation_cycle_64x256b`;
- `request_scratch_pool_cycle_16x64x256b`.

These benchmarks exercise the base arena fast path directly and through request arena reuse.

## Expected Result

The public `locus_alloc::*` scratch arena API should remain source compatible, the root allocator file should shrink, unit tests should keep passing, and real scratch allocation benchmarks should continue to run successfully.
