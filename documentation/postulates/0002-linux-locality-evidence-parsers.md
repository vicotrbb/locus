# Postulate 0002: Linux Locality Evidence Parsers

Date: 2026-07-02

## Statement

Locus needs parsers for Linux locality evidence before it introduces NUMA memory-policy syscalls, because allocator experiments should be validated against observable page placement and node counters.

## Rationale

The research notes identify `/proc/<pid>/numa_maps`, `/sys/devices/system/node/node*/numastat`, and cgroup v2 `memory.numa_stat` as key evidence sources. Parsing those formats first creates a safe validation layer that later allocator experiments can reuse.

## Experiment

Implement a `locus-observe` crate with parsers for:

- `/proc/<pid>/numa_maps`;
- cgroup v2 `memory.numa_stat`;
- `/sys/devices/system/node/node*/numastat`.

Add fixture-style unit tests for representative lines and invalid input.

## Expected Result

The parsers should pass tests on non-Linux development hosts because they operate on supplied text. The result should make later Linux-only allocation experiments easier to validate.
