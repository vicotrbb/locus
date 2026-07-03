# Postulate 0171: Remote-Free Positive Drift Matrix

Date: 2026-07-03

## Claim

`RemoteFreeQueuedByteDriftReport` should be validated against deliberately
mis-sized real allocation traces, not only zero-drift unit tests and the
known-good queued-byte config.

## Rationale

Experiment 0178 proved that the mixed-size queued-byte config reports zero
pending drift, zero queued-byte drift, and zero queue backpressure on the real
allocation path. That is necessary but incomplete evidence.

Adaptive remote-free policy work needs positive signal evidence. A report that
only survives matching configs could still fail to expose a too-small pending
window, a too-low byte budget, or producer backpressure when those cases appear
in a real owner loop.

## Experiment

Add a focused `remote_free_drift_matrix` benchmark with real `Vec` allocation
blocks, `RemoteFreeQueue`, `RemoteFreeDrainController`, and
`RemoteFreeQueuedByteDriftReport`.

The matrix should include:

- a matched end-drain config with no drift;
- a too-small pending target with pending drift and no queued-byte drift;
- a too-low queued-byte budget with queued-byte drift and no pending drift;
- a small queue capacity case that records queue backpressure.

Keep this benchmark separate from `remote_free_mixed_size_policy.rs` so the
policy benchmark does not become a large mixed-purpose file.

## Falsification

The postulate is weakened if positive drift cases do not produce non-zero
diagnostic counters, if the backpressure case does not observe `full_count`, if
the benchmark avoids real allocations, or if adding the matrix requires policy
mutation.

## Expected Value

If the postulate survives, adaptive policy work can rely on drift diagnostics
that have been tested against both matching configs and deliberately bad config
inputs on real remote-free allocation paths.
