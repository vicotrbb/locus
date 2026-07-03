# Postulate 0127: Mapped Scratch Module

Date: 2026-07-03

## Statement

The mmap-backed `MappedScratchArena` should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

`MappedScratchArena` is the allocator-facing bridge from safe domain allocation to mapped memory, page touching, page locking, NUMA binding, and transparent huge page advice. Keeping this object in `src/lib.rs` mixes a system-backed allocator and its Linux policy wrappers with parser code, pinned pool logic, KV-cache allocation, remote-free infrastructure, and request scratch managers.

Moving the mapped scratch arena into `mapped_scratch.rs` should reduce root file size, isolate mmap-backed allocation and Linux advice behavior, and keep the public API source compatible through root re-exports.

## Experiment

Extract the mapped scratch arena subsystem into `crates/locus-alloc/src/mapped_scratch.rs`.

The module should own:

- `MappedScratchArena`;
- `MappedScratchAllocError`;
- `MappedScratchHugePageAdvice`;
- private alignment rounding for the mapped arena;
- focused tests for alignment, reset accounting, out-of-memory behavior, page touching, page locking, mapping identity, Linux bind rejection, and transparent huge page advice.

The pinned scratch pool and mapped scratch output parsers can remain in the root for this experiment. Any helper required by the pinned pool should remain crate-private and should not become part of the public API.

## Real Workload Gate

The extraction must still pass existing mapped scratch validation paths:

- `mapped_scratch_arena_reset_cycle_64x256b`;
- `mapped_scratch_write_touch_1mib`;
- `mapped_scratch_write_touch_4mib_*`;
- `mapped_scratch_lock` example in Docker;
- `mapped_scratch_thp` example in Docker.

These exercise mapped allocation, page materialization, page locking behavior, Linux advice behavior, and stable observability output.

## Expected Result

The public `locus_alloc::*` mapped scratch API should remain source compatible, the root allocator file should shrink, unit tests should keep passing, Docker examples should still produce stable mapped scratch output, and real mapped scratch benchmarks should continue to run successfully.
