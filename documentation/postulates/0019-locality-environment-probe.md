# Postulate 0019: Locality Environment Probe

Date: 2026-07-02

## Statement

A combined observability example should make live NUMA validation readiness easier to inspect than running each evidence source separately.

## Rationale

Placement validation needs multiple independent signals. `numa_maps`, cgroup `memory.numa_stat`, and per-node `numastat` each fail differently in containers, kernels, and delegated cgroups. A compact probe should report availability across all three without treating unavailable counters as allocator failures.

## Experiment

Add a `locality_environment` example that:

- reads `/proc/self/numa_maps` and prints summary availability;
- resolves the current cgroup v2 `memory.numa_stat` path and prints summary availability;
- scans `/sys/devices/system/node/node*/numastat` and prints summary availability;
- returns explicit unavailable lines for missing optional Linux evidence.

## Expected Result

The example should compile under all-target checks. In the current Docker validation environment it should report all three evidence sources as unavailable.
