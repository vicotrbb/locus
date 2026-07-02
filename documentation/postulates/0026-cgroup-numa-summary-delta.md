# Postulate 0026: Cgroup Numa Summary Delta

Date: 2026-07-02

## Statement

Locus should compute signed before-and-after deltas for cgroup `memory.numa_stat` summaries to support secondary placement evidence around allocation probes.

## Rationale

`memory.numa_stat` reports current byte totals by node and metric. A validation run that samples before and after an allocation needs signed deltas to distinguish bytes added to the expected node from bytes removed or moved elsewhere. Encoding this in `locus-observe` keeps probe code simple and tested.

## Experiment

Add a `CgroupNumaDelta` helper that:

- computes signed total-byte deltas;
- computes signed per-node byte deltas;
- includes nodes that appear only before or only after;
- passes fixture tests with positive, negative, and new-node deltas.

## Expected Result

The helper should pass workspace tests and clippy. It should not treat cgroup deltas as primary placement proof, only as secondary evidence.
