# Postulate 0006: Locality Evidence Summaries

Date: 2026-07-02

## Statement

Allocator experiments need shared summary types for locality evidence so page-placement and cgroup NUMA results can be compared consistently across experiments.

## Rationale

Parsing raw Linux observability files is only the first step. Experiment logs need concise totals such as mappings inspected, pages by NUMA node, and bytes by NUMA node. If every experiment computes those by hand, the risk of inconsistent interpretation grows.

## Experiment

Add summary types for:

- parsed `/proc/<pid>/numa_maps` entries;
- parsed cgroup v2 `memory.numa_stat` entries.

Validate totals and per-node aggregation with fixture tests.

## Expected Result

The summary layer should preserve raw parser access while giving allocator experiments a stable reporting surface.
