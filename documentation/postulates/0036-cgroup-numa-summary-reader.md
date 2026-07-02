# Postulate 0036: Cgroup Numa Summary Reader

Date: 2026-07-02

## Statement

`locus-observe` should provide a shared reader that returns a summarized cgroup v2 `memory.numa_stat` view from an explicit path.

## Rationale

Several examples and probes read cgroup NUMA stats and immediately convert them into `CgroupNumaSummary`. A shared helper reduces duplication and keeps future validation probes focused on evidence interpretation instead of parser plumbing.

## Experiment

Add a `read_cgroup_numa_summary` helper that:

- reads an explicit `memory.numa_stat` path;
- parses the entries;
- returns `CgroupNumaSummary`;
- reuses existing read and parse errors;
- passes fixture coverage in the observability file reader test.

## Expected Result

The helper should pass workspace tests and clippy without changing parser semantics.
