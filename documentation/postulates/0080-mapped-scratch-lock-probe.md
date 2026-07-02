# Postulate 0080: Mapped Scratch Lock Probe

Date: 2026-07-02

## Statement

Locus should provide a small executable probe for mapped scratch page locking.

## Rationale

The new `MappedScratchArena::lock_pages` method is useful only if its environment behavior can be observed outside unit tests. A probe that prints stable lock and unlock lines lets Docker and future host runs record whether page locking is permitted.

This is a precursor to pinned host staging buffers. It does not claim CUDA host registration or GPU DMA readiness.

## Experiment

Add a `mapped_scratch_lock` example to `locus-alloc` that:

- creates a mapped scratch arena;
- prints mapping identity;
- write-touches pages;
- calls `lock_pages`;
- reports `page_lock=ok` or `page_lock=error <error>`;
- calls `unlock_pages` after a successful lock and reports `page_unlock=ok` or `page_unlock=error <error>`.

Run the example in Docker and record the output.

## Expected Result

The probe should compile and print explicit page-lock status. In the current Docker environment, a small arena is expected to lock and unlock successfully.
