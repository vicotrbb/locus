# Postulate 0020: First Touch Materialization Benchmark

Date: 2026-07-02

## Statement

Locus should measure page materialization cost separately from arena allocation mechanics before using mapped arenas for NUMA placement claims.

## Rationale

Linux NUMA policy only becomes meaningful after pages are faulted into memory. Earlier mapped scratch benchmarks measure reset-cycle allocation metadata after a mapping exists, but they do not measure the cost of write-touching pages. A first-touch benchmark helps separate mapping creation, page fault materialization, and later allocator fast-path behavior.

## Experiment

Extend the existing Criterion harness with:

- `mapped_scratch_write_touch_1mib`, which creates a 1 MiB mapped scratch arena and write-touches one byte per page;
- `vec_write_touch_1mib`, which allocates a 1 MiB `Vec<u8>` and writes one byte per 4 KiB page as a default allocator baseline.

## Expected Result

The benchmark should compile under all-target checks and produce repeatable first-touch measurements. The mapped path is expected to cost more than arena reset allocation because it includes mapping and page fault work.
