# Placement Proof Checklist

Date: 2026-07-02

## Purpose

Locus now has the pieces needed to attempt NUMA placement validation for a mapped scratch arena, but the current Docker environment still blocks successful proof. This note defines the evidence required before any result is described as verified NUMA placement.

## Required Primary Evidence

A placement proof for one mapped scratch arena requires all of the following:

- the probe prints the arena mapping start and length;
- Linux memory policy application succeeds for the expected node;
- the arena pages are write-touched after policy application;
- `/proc/self/numa_maps` is readable;
- the mapped arena address matches a `numa_maps` entry by exact start or containing range;
- `NumaPlacementEvidence::is_fully_on_expected_node()` returns true;
- `placement_verified=true` is printed for that matched mapping.
- `placement_proof=verified reason=verified` is printed for the attempt.

If any item is missing, the result is attempted placement or unavailable evidence, not verified placement.

## Secondary Evidence

Secondary signals should be recorded alongside the primary proof when available:

- cgroup `memory.numa_stat` before-and-after byte deltas;
- node `numastat` before-and-after metric deltas;
- node and cgroup unavailable states when the environment does not expose the counters.

Secondary counters are useful for correlation, but they do not prove placement for a specific mapping without the `numa_maps` match.

## Current Docker Result

The current Docker validation path still reports:

- `mbind` fails with `Operation not permitted`;
- `cgroup_numa_delta=unavailable`;
- `node_numastat_delta=unavailable`;
- `numa_maps=unavailable`.
- `placement_proof=unavailable reason=numa_maps_unavailable`.

That environment validates failure handling and probe plumbing. It does not validate successful NUMA placement.

## Next Validation Environment

The next meaningful validation run needs a Linux host or container configuration where:

- `mbind` is permitted;
- `/proc/self/numa_maps` is readable;
- cgroup v2 `memory.numa_stat` is exposed for the current process cgroup;
- `/sys/devices/system/node/node*/numastat` is readable.

Only then can the mapped scratch bind probe produce a placement proof rather than an unavailable-evidence report.
