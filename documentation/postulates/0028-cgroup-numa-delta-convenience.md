# Postulate 0028: Cgroup Numa Delta Convenience

Date: 2026-07-02

## Statement

Cgroup NUMA deltas should expose small query helpers so validation probes can check node deltas and no-op snapshots without duplicating map logic.

## Rationale

The delta structure is intended for probe output and later assertions. Callers will commonly ask for one node's signed delta or whether any delta is non-zero. Keeping those operations in `locus-observe` makes future validation code simpler and consistent.

## Experiment

Add methods that:

- return the signed byte delta for one node;
- report whether any aggregate or node delta is non-zero;
- pass fixture tests for positive deltas and no-op snapshots.

## Expected Result

The helpers should pass workspace tests and clippy without changing delta semantics.
