# Postulate 0072: Placement Readiness Consistency

Date: 2026-07-02

## Statement

The NUMA placement validation readiness parser should reject inconsistent status and reason pairs.

## Rationale

Placement readiness determines whether the environment has enough evidence sources to attempt a placement proof. A contradictory line such as `placement_validation_readiness=ready reason=numa_maps_unavailable` should not be accepted as valid readiness evidence.

Rejecting inconsistent pairs keeps the validation gate from combining malformed probe output.

## Experiment

Add consistency rules for `NumaPlacementValidationReadiness`:

- `ready` must use `ready`;
- `not_ready` must use `numa_maps_unavailable`, `cgroup_numa_stat_unavailable`, or `node_numastat_unavailable`.

Update line and output parser tests to reject inconsistent pairs.

## Expected Result

The parser should continue accepting valid placement readiness output and reject inconsistent status and reason combinations. Workspace validation should pass.
