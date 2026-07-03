# Postulate 0098: Mapped Scratch THP Probe

Date: 2026-07-02

## Statement

`MappedScratchArena` transparent huge page advice should have a runnable Linux probe with page-size evidence.

## Rationale

The allocator now exposes opt-in THP advice, but accepted `madvise` does not prove huge page adoption. A probe should apply the hint to a mapped scratch arena, write-touch the arena, and inspect live `numa_maps` evidence for the tested mapping.

This keeps the distinction clear:

- `thp_advice=ok` means the kernel accepted the advisory request;
- `kernel_page_kb=<n>` reports observed mapping evidence when available;
- huge page adoption is not claimed unless evidence shows a page size larger than the base page size.

## Expected Result

Add a `mapped_scratch_thp` example to `locus-alloc`. On Linux it should accept optional `hugepage` or `no_hugepage` mode, apply the advice, touch pages, then print stable mapping and `numa_maps` evidence lines. On non-Linux targets it should print `mapped_scratch_thp=unsupported-platform`.
