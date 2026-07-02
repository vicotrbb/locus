# Postulate 0027: Bind Probe Cgroup Delta

Date: 2026-07-02

## Statement

The mapped scratch bind probe should sample cgroup `memory.numa_stat` before and after page touch when available, then print signed NUMA byte deltas as secondary evidence.

## Rationale

`numa_maps` remains the primary evidence for a specific mapping. Cgroup NUMA bytes provide a useful process-container level signal around the same allocation event, especially in production-like cgroups. The probe should collect that signal opportunistically without failing in containers that do not expose it.

## Experiment

Update the mapped scratch bind probe to:

- resolve the current cgroup v2 `memory.numa_stat` path;
- sample a cgroup NUMA summary before page touch;
- sample again after page touch;
- print signed total and per-node deltas when both samples are available;
- print an explicit unavailable line otherwise.

## Expected Result

The probe should compile under all-target checks. In the current Docker environment, cgroup NUMA deltas should report unavailable.
