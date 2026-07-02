# Postulate 0015: Mapped Scratch Linux Bind

Date: 2026-07-02

## Statement

Mapped scratch arenas should expose a Linux-only bind method so allocator experiments can apply NUMA placement policy without calling the system boundary directly.

## Rationale

The `locus-sys` `mbind` wrapper proves the syscall path exists, but allocator experiments should operate on allocator objects. A `MappedScratchArena::bind_to_node` method keeps policy application connected to the mapped arena under test while preserving the narrow unsafe boundary.

## Experiment

Add a Linux-only `MappedScratchArena::bind_to_node` method that:

- accepts a `NodeId`;
- calls `locus-sys` `bind_region_to_node`;
- wraps Linux policy errors in `MappedScratchAllocError`;
- has a Linux-only test for invalid node handling that does not require syscall permission.

## Expected Result

The method should pass local workspace gates through conditional compilation and pass Linux container tests for the invalid-node path.
