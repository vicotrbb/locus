# Postulate 0096: Linux Transparent Huge Page Advice

Date: 2026-07-02

## Statement

`locus-sys` should expose a safe Linux-only helper for transparent huge page advice on owned mapped regions.

## Rationale

The research notes identify huge pages as an opt-in backing-store policy, not a default allocator behavior. The narrowest first step is to keep `madvise` inside `locus-sys` and attach `MADV_HUGEPAGE` or `MADV_NOHUGEPAGE` to a `MappedRegion` owned by Locus.

This helper only requests kernel advice. It must not claim that the region was promoted to huge pages. Promotion still requires observability through `numa_maps`, `smaps`, or related kernel counters.

## Expected Result

Add a Linux-only transparent huge page advice enum and function for `MappedRegion`. Docker Linux tests should verify that both advice modes can be issued against an anonymous mapping in the current container.
