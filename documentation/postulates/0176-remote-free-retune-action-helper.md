# Postulate 0176: Remote-Free Retune Action Helper

Date: 2026-07-03

## Claim

`RemoteFreeQueuedByteDriftReport` should expose a typed retune-action helper
that maps observed drift signals to the next benchmark candidate without
duplicating ad hoc decision logic in benchmark files.

## Rationale

Experiments 0181 through 0183 separated three cases:

- queue backpressure without retained-window drift can start with capacity
  validation;
- retained pending and queued-byte drift should test earlier owner drains;
- backpressure plus retained-window drift should test capacity plus earlier
  owner drains.

The current benchmarks encode those decisions as expected labels. A reusable
helper would make runtime-facing callers and future benchmarks use the same
classification.

## Experiment

Add a public typed retune action and expose it from
`RemoteFreeQueuedByteDriftReport`. Validate every signal combination with unit
tests, then wire the uniform and mixed-size retune benchmarks to print and
assert the recommended action while exercising real allocation and queue
activity.

## Falsification

The postulate is weakened if the helper cannot distinguish capacity-only
backpressure from retained-window drift, if it recommends capacity alone for
the known capacity-only drift cases from experiments 0182 and 0183, or if the
benchmarks still need separate hard-coded decision logic to interpret the same
drift report.

## Expected Value

If the postulate survives, the retune path moves from diagnostic labels toward
a reusable runtime-facing recommendation while keeping the action explicitly
benchmark-first and non-mutating.
