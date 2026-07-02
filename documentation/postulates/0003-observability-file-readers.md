# Postulate 0003: Observability File Readers

Date: 2026-07-02

## Statement

Parser-only locality evidence is not enough for later allocator validation. Locus should expose file readers that connect the safe parsers to explicit Linux observability paths, while still allowing fixture-based tests on non-Linux hosts.

## Rationale

Allocator experiments need to record raw commands, summarized outputs, and conclusions. If Locus can read `numa_maps`, cgroup `memory.numa_stat`, and node `numastat` from explicit paths, later experiments can validate page placement without duplicating file IO and parse handling.

## Experiment

Add reader functions to `locus-observe` for:

- explicit `numa_maps` paths;
- `/proc/self/numa_maps`;
- explicit cgroup `memory.numa_stat` paths;
- explicit node `numastat` paths.

Validate the readers with temporary fixture files.

## Expected Result

The readers should pass fixture tests on this host and return ordinary read errors for missing Linux-only files.
