# Postulate 0031: Bind Probe Node Numastat Delta

Date: 2026-07-02

## Statement

The mapped scratch bind probe should sample system node `numastat` snapshots before and after page touch when available, then print per-node metric deltas as secondary locality evidence.

## Rationale

`numa_maps` is the primary mapping-specific evidence, while cgroup and node counters are secondary signals. Node `numastat` deltas can show whether visible NUMA node counters moved around the probe window. The probe should collect that signal opportunistically without failing when node sysfs counters are unavailable.

## Experiment

Update the mapped scratch bind probe to:

- sample a system node `numastat` snapshot before page touch;
- sample again after page touch;
- print selected per-node metric deltas when both samples are available;
- print an explicit unavailable line otherwise.

## Expected Result

The probe should compile under all-target checks. In the current Docker environment, node `numastat` deltas should report unavailable.
