# Postulate 0097: Mapped Scratch Transparent Huge Page Advice

Date: 2026-07-02

## Statement

`MappedScratchArena` should expose opt-in transparent huge page advice without leaking syscall details to allocator users.

## Rationale

`locus-sys` now owns the Linux `madvise` boundary for `MADV_HUGEPAGE` and `MADV_NOHUGEPAGE`. The allocator-facing API should attach that hint to `MappedScratchArena`, because the arena is the memory object future request, KV, and pinned staging pools will build on.

This remains a hint, not a proof. Successful advice only means the kernel accepted the request. Huge page adoption still requires separate observability.

## Expected Result

Add a Linux-only `MappedScratchHugePageAdvice` API on `MappedScratchArena`, wrap advice failures in `MappedScratchAllocError`, and validate it with Docker Linux tests.
