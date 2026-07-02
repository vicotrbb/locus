# Postulate 0010: Owned Mapped Region

Date: 2026-07-02

## Statement

An owned anonymous mapped region is the smallest useful unsafe substrate for moving Locus from Vec-backed experiments toward real memory placement validation.

## Rationale

Linux memory-policy validation applies to virtual address ranges and physical pages created from mappings. Before introducing `mbind` or page residency checks, Locus needs a safe owned wrapper around an anonymous mapping with deterministic cleanup.

## Experiment

Add `locus-sys` with:

- a local unsafe-code lint override;
- `MappedRegion::anonymous`;
- safe shared and mutable slice access;
- deterministic `munmap` cleanup on drop;
- focused tests for writable mappings and invalid length rejection.

## Expected Result

The crate should compile under the workspace gates, and the rest of the workspace should remain unsafe-forbidden by default.
