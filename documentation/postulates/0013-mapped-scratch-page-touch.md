# Postulate 0013: Mapped Scratch Page Touch

Date: 2026-07-02

## Statement

Mapped scratch arenas should expose an explicit page-touch operation so future locality experiments can materialize pages before inspecting placement.

## Rationale

The system boundary can write-touch mapped regions, but allocator experiments should not need to know about page-size discovery or raw mapped-region details. A safe arena-level method keeps locality validation attached to the allocator abstraction under test.

## Experiment

Add `MappedScratchArena::write_touch_pages` that:

- discovers page size through `locus-sys`;
- write-touches the arena mapping;
- returns the number of touched page strides;
- converts system errors into the mapped scratch error type;
- has focused unit coverage.

## Expected Result

The method should pass the workspace gates and make mapped scratch arenas ready for future `numa_maps` and memory-policy experiments.
