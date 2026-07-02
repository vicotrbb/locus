# Linux Observability Environment Requirements

Date: 2026-07-02

## Context

The current allocator foundation can create owned anonymous mappings, write-touch pages, attempt Linux `mbind`, and parse Linux locality evidence from:

- `/proc/self/numa_maps`;
- cgroup v2 `memory.numa_stat`;
- `/sys/devices/system/node/node*/numastat`.

These signals are necessary before claiming that a mapped scratch arena was actually placed on a requested NUMA node.

## Current Docker Evidence

The current Docker validation environment is useful for API and failure-path coverage, but it is not sufficient to prove NUMA placement:

- `mbind` returns `Operation not permitted`.
- `/proc/self/numa_maps` is unavailable.
- cgroup `memory.numa_stat` is unavailable for the current process cgroup.
- node `numastat` files are unavailable.

This means current Linux container runs validate graceful failure handling, parsing, examples, and unsafe boundary behavior. They do not validate successful node binding.

## Placement Proof Requirement

A successful placement proof should include:

- a permitted `mbind` or equivalent NUMA policy operation;
- write-touching the mapped pages after policy application;
- `/proc/self/numa_maps` evidence showing pages on the expected node;
- cgroup or node-level NUMA counters as secondary evidence;
- a recorded comparison against the default placement path.

Until those signals are available in a host or privileged test environment, allocator placement results must be described as attempted policy application rather than verified NUMA placement.
