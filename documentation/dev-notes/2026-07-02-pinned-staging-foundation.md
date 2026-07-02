# Pinned Staging Foundation

Date: 2026-07-02

## Purpose

Locus now has the first low-level primitive needed for future GPU-near pinned host staging buffers: safe page locking for owned mapped memory.

## Current Foundation

- `MappedRegion::lock_pages` wraps `mlock`.
- `MappedRegion::unlock_pages` wraps `munlock`.
- `MappedScratchArena::lock_pages` and `MappedScratchArena::unlock_pages` expose the same behavior at the allocator layer.
- `cargo run -p locus-alloc --example mapped_scratch_lock` prints mapping identity, page-touch count, and page-lock status.

The current Docker probe reports:

```text
mapping_start=0xffff8367a000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
```

## Non-Goals So Far

This foundation does not yet provide:

- CUDA host registration;
- GPU DMA readiness proof;
- a budgeted pinned staging pool;
- per-GPU buffer accounting;
- NUMA placement proof for locked pages;
- C ABI or framework integration.

## Next Questions

- Should pinned staging buffers use `MappedScratchArena` directly or a separate pool type with explicit checkout and return?
- What budget policy should cap locked bytes per process, per GPU, and per NUMA node?
- Should page-lock readiness become a parsed validation gate line before allocator integration?
- How should GPU locality metadata from PCI topology lower into pinned staging allocation decisions?
