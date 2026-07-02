# Postulate 0018: Current Cgroup Numa Stat Path

Date: 2026-07-02

## Statement

Live cgroup NUMA observability should resolve the current process cgroup path from `/proc/self/cgroup` instead of assuming the cgroup v2 root.

## Rationale

Production inference services often run inside nested cgroups. Reading only `/sys/fs/cgroup/memory.numa_stat` works for a root cgroup but misses delegated or nested cgroup paths. A pure resolver makes examples and later validation tools representative of real deployment layouts.

## Experiment

Add a resolver that:

- parses the unified cgroup v2 `0::` entry;
- joins it with a configurable cgroup root;
- returns the `memory.numa_stat` path;
- rejects inputs without a unified cgroup v2 entry;
- updates the live cgroup example to use the resolver.

## Expected Result

The resolver should pass fixture tests and the Docker example should continue to report unavailable state when `memory.numa_stat` is not exposed.
