# Postulate 0012: Write-Touch Mapped Pages

Date: 2026-07-02

## Statement

Locus needs an explicit write-touch primitive for mapped regions before page-placement experiments can be trusted.

## Rationale

NUMA policy and page locality are applied when pages are faulted or migrated, not merely when virtual address ranges are reserved. A mapped arena that has not been touched may not have physical pages to inspect. A safe write-touch helper lets allocator experiments materialize pages without exposing raw pointers outside `locus-sys`.

## Experiment

Add to `locus-sys`:

- `page_size`;
- `MappedRegion::write_touch_pages`;
- tests for page-size discovery;
- tests that one byte per page stride is touched;
- invalid page-size rejection.

## Expected Result

The helper should pass local and Linux container tests, and it should become the building block for later `numa_maps` validation around mapped arenas.
