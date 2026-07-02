# Postulate 0083: Pinned Scratch Pool

Date: 2026-07-02

## Statement

Locus should expose a small budgeted pinned scratch pool backed by page-locked mapped scratch arenas.

## Rationale

The page-lock primitive proves that small mapped arenas can be locked, but future GPU staging work needs reuse and budget enforcement. A pool that locks arenas once and reuses them keeps pinning explicit while avoiding ad hoc lock and unlock calls at transfer sites.

This remains host-only pinned memory. It does not register buffers with CUDA, bind them near a GPU, or model async stream completion.

## Experiment

Add a `PinnedScratchPool` that:

- owns page-locked `MappedScratchArena` instances;
- enforces a maximum locked-byte budget;
- returns opaque checkout handles;
- allows mutable arena access by handle;
- resets and reuses arenas on release;
- reports basic pool accounting.

Add focused tests for budget enforcement, reuse, invalid handles, and allocation through a checked-out arena.

## Expected Result

The pool should compile as a safe allocator-layer primitive. Host and Docker `locus-alloc` tests should pass when small page locks are permitted.
