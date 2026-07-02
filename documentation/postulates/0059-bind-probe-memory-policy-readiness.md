# Postulate 0059: Bind Probe Memory Policy Readiness

Date: 2026-07-02

## Statement

The mapped scratch bind probe should print the typed Linux memory-policy readiness verdict for its own `bind_to_node` attempt.

## Rationale

The standalone `mbind_region` probe now reports whether Linux memory policy application is ready, but the allocator-level bind probe is the path used for placement proof attempts. A successful placement proof requires the allocator policy attempt to succeed before `numa_maps` evidence can prove page location.

Printing the same memory-policy readiness line from `mapped_scratch_bind` keeps allocator-level validation output self-contained.

## Experiment

Update the mapped scratch bind probe to derive `memory_policy_readiness=<status> reason=<reason>` from the `MappedScratchArena::bind_to_node` result.

## Expected Result

The probe should preserve its existing `mapped_scratch_bind` and `placement_proof` output. In Docker it should also report `memory_policy_readiness=not_ready reason=permission_denied`.
