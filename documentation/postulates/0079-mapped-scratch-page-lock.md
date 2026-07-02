# Postulate 0079: Mapped Scratch Page Lock

Date: 2026-07-02

## Statement

`MappedScratchArena` should expose safe page-lock and unlock operations by delegating to its owned mapped region.

## Rationale

Future GPU-near pinned staging buffers need allocator-level APIs, not direct access to `locus-sys`. The mapped scratch arena already owns the memory region used by placement experiments, so it is the smallest allocator object that can expose safe page-lock behavior while preserving the narrow unsafe boundary.

This remains a staging primitive. It does not yet implement a budgeted pinned pool, GPU registration, or CUDA/NVML integration.

## Experiment

Add `MappedScratchArena::lock_pages` and `MappedScratchArena::unlock_pages`. Wrap page-locking errors in `MappedScratchAllocError`, then add focused tests.

Run host validation and Docker `locus-alloc` tests.

## Expected Result

Small mapped scratch arenas should lock and unlock successfully where the environment permits `mlock`. If an environment denies locking, callers should receive an explicit page-locking error.
