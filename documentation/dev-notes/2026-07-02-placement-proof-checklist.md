# Placement Proof Checklist

Date: 2026-07-02

## Purpose

Locus now has the pieces needed to attempt NUMA placement validation for a mapped scratch arena, but the current Docker environment still blocks successful proof. This note defines the evidence required before any result is described as verified NUMA placement.

## Required Primary Evidence

A placement proof for one mapped scratch arena requires all of the following:

- the probe prints the arena mapping start and length;
- Linux memory policy application succeeds for the expected node;
- `memory_policy_readiness=ready reason=ready` is printed for the allocator policy attempt;
- the arena pages are write-touched after policy application;
- `/proc/self/numa_maps` is readable;
- the mapped arena address matches a `numa_maps` entry by exact start or containing range;
- `NumaPlacementEvidence::is_fully_on_expected_node()` returns true;
- `placement_verified=true` is printed for that matched mapping;
- `placement_proof=verified reason=verified` is printed for the attempt.
- `placement_validation_gate=verified reason=verified` is printed when the three probe outputs are evaluated together.

If any item is missing, the result is attempted placement or unavailable evidence, not verified placement.

## Secondary Evidence

Secondary signals should be recorded alongside the primary proof when available:

- cgroup `memory.numa_stat` before-and-after byte deltas;
- node `numastat` before-and-after metric deltas;
- node and cgroup unavailable states when the environment does not expose the counters.

Secondary counters are useful for correlation, but they do not prove placement for a specific mapping without the `numa_maps` match.

## Readiness Gates

Before a validation run is treated as capable of proving placement, record both readiness probes:

- `memory_policy_readiness=ready reason=ready` from the Linux memory-policy path;
- `placement_validation_readiness=ready reason=ready` from the locality evidence environment path.

If either readiness line reports `not_ready`, the run can still validate failure handling and parser plumbing, but it cannot produce a successful placement proof.

The preferred one-command validation path is `cargo run -p locus-validate --example live_placement_validation_gate`. Captured probe outputs can also be evaluated with `cargo run -p locus-validate --example placement_validation_gate -- <memory-policy-output> <placement-readiness-output> <placement-proof-output>`. A verified run requires `placement_validation_gate=verified reason=verified`.

## Current Docker Result

The current Docker validation path still reports:

- `mbind` fails with `Operation not permitted`;
- `memory_policy_readiness=not_ready reason=permission_denied`;
- `seccomp=filter seccomp_filters=1 no_new_privs=0`;
- `cgroup_numa_delta=unavailable`;
- `node_numastat_delta=unavailable`;
- `numa_maps=unavailable`;
- `placement_proof=unavailable reason=numa_maps_unavailable`.

The locality environment probe also reports:

- `placement_validation_readiness=not_ready reason=numa_maps_unavailable`.

The combined validation gate reports:

- `placement_validation_gate=not_ready reason=memory_policy_not_ready`.

That environment validates failure handling and probe plumbing. It does not validate successful NUMA placement.

## Next Validation Environment

The next meaningful validation run needs a Linux host or container configuration where:

- `mbind` is permitted;
- seccomp is disabled or configured to allow the NUMA policy syscalls required by the probe;
- the memory-policy readiness line reports `ready`;
- `/proc/self/numa_maps` is readable;
- cgroup v2 `memory.numa_stat` is exposed for the current process cgroup;
- `/sys/devices/system/node/node*/numastat` is readable;
- the placement validation readiness line reports `ready`;
- the combined validation gate reports `verified`.

Only then can the mapped scratch bind probe produce a placement proof rather than an unavailable-evidence report.
