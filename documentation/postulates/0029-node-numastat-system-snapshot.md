# Postulate 0029: Node Numastat System Snapshot

Date: 2026-07-02

## Statement

Locus should represent node `numastat` snapshots across NUMA nodes as one system-level artifact with signed per-node deltas.

## Rationale

The current node `numastat` support parses and snapshots one node at a time. Placement validation often needs a before-and-after view across all visible nodes so local and remote node counter movement can be compared without ad hoc maps in probe code.

## Experiment

Add system-level node `numastat` helpers that:

- hold per-node `NodeNumastatSnapshot` values;
- report the number of inspected nodes;
- compute signed per-node metric deltas;
- include nodes that appear only before or only after;
- pass fixture tests for changed, removed, and new nodes.

## Expected Result

The helpers should pass workspace tests and clippy. They should provide secondary locality evidence, not primary proof for a specific mapping.
