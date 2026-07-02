# Postulate 0017: Node Numastat Snapshots

Date: 2026-07-02

## Statement

Allocator placement experiments need reusable node `numastat` snapshots and deltas so before-and-after locality counters can be recorded consistently.

## Rationale

The parser layer can read `numastat`, but experiments need more than raw rows. Counters such as `numa_hit`, `numa_miss`, `local_node`, and `other_node` are useful when captured before and after an allocation workload. A shared snapshot and delta type reduces duplicated aggregation logic.

## Experiment

Add:

- `NodeNumastatSnapshot`;
- signed `NodeNumastatDelta`;
- fixture tests for metric lookup and before-after deltas.

## Expected Result

The snapshot layer should remain pure and fixture-testable while preparing the project for live node-counter experiments.
