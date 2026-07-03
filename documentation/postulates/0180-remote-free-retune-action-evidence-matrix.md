# Postulate 0180: Remote-Free Retune Action Evidence Matrix

Date: 2026-07-03

## Claim

Before adding adaptive remote-free policy behavior, Locus should encode the
validated `RemoteFreeQueuedByteRetuneAction` evidence as a named matrix that
spans the generic, mixed-size, owner-loop, KV-cache, and request-affine arena
surfaces.

## Rationale

Experiments 0184 through 0187 validated the same diagnostic action helper
against multiple remote-free workloads. The evidence now covers generic
uniform traces, heterogeneous traces, a runtime-facing owner-loop example, real
KV block handles, and real request scratch arenas. Adaptive policy will depend
on this mapping, so the validated cases should be easy to review and should
fail loudly if future changes alter the action semantics.

## Experiment

Add a focused public-API test that names the validated surfaces and rebuilds
their drift reports from queued-byte configs, queue stats, and controller
observations.

The matrix should assert:

- capacity-only generic traces with only queue backpressure recommend
  `increase_queue_capacity`;
- capacity-only generic and mixed-size traces with retained-window drift and
  remaining backpressure recommend
  `increase_queue_capacity_and_drain_earlier`;
- end-drain generic, mixed-size, KV, and request traces with retained-window
  drift but no queue backpressure recommend `drain_earlier`;
- policy-drained generic, mixed-size, owner-loop, KV, and request traces
  recommend `keep_config`.

## Falsification

The postulate fails if the matrix cannot be represented with the public
queued-byte config and drift-report APIs, if any validated surface maps to a
different action, or if the added test obscures the distinction between
capacity backpressure, retained-byte drift, and clean policy-drained windows.

## Expected Value

If the postulate survives, the project has a compact regression tripwire for
the action mapping that future adaptive policy work can cite before mutating
queue capacity or owner drain cadence.
