# Postulate 0037: Shared Observability Readers In Probes

Date: 2026-07-02

## Statement

Live examples and probes should use shared observability readers instead of duplicating summary construction and sysfs scanning logic.

## Rationale

The observability crate now owns cgroup summary reading and system node `numastat` reading. Routing examples through those helpers reduces drift between validation tools and keeps future probes focused on interpreting evidence.

## Experiment

Update:

- the cgroup NUMA example to use `read_cgroup_numa_summary`;
- the mapped scratch bind probe to use `read_cgroup_numa_summary`;
- the locality environment probe to use both shared cgroup summary and node system snapshot readers.

## Expected Result

The examples should compile under all-target checks. Docker probes should preserve their current unavailable-state output.
