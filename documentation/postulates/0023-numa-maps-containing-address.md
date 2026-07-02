# Postulate 0023: Numa Maps Containing Address

Date: 2026-07-02

## Statement

Placement validation should support matching a probed address to the ordered `numa_maps` entry that contains it, not only to an exact mapping start.

## Rationale

Exact start matching is useful for anonymous mappings, but validation tools often inspect an address inside a range. `/proc/<pid>/numa_maps` reports VMA start addresses in order, so the next start can be used as the effective end of the current entry for lookup purposes.

## Experiment

Add:

- a `numa_maps_entry_containing_address` helper;
- focused parser tests for interior, missing, and final-entry addresses;
- a fallback in the mapped scratch bind probe when exact start matching fails.

## Expected Result

The helper should pass workspace tests and clippy. The current Docker bind probe should still report unavailable `numa_maps` evidence, but future readable environments can distinguish exact and containing matches.
