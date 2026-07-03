# Postulate 0172: Remote-Free Drift Retune Hint

Date: 2026-07-03

## Claim

`RemoteFreeQueuedByteDriftReport` should expose a typed retune hint that
classifies the first diagnostic response to observed drift without mutating the
remote-free drain policy.

## Rationale

Experiment 0179 proved that the drift report can identify positive pending
drift, queued-byte drift, and queue backpressure on real allocation paths.
Callers still need to translate those signals into candidate actions.

Keeping that translation as a small typed diagnostic reduces duplicated
benchmark-local logic while avoiding an unvalidated adaptive policy.

## Experiment

Add a public `RemoteFreeQueuedByteRetuneHint` enum with these diagnostic
outcomes:

- keep the config when no drift is observed;
- increase queue capacity when backpressure is the only signal;
- review drain cadence when pending items exceed the target window;
- review queued-byte budget when retained bytes exceed the budget;
- review multiple signals when more than one drift signal is present.

Expose `RemoteFreeQueuedByteDriftReport::retune_hint()` and print the hint from
the positive drift matrix benchmark.

## Falsification

The postulate is weakened if the hint mutates policy, hides multiple drift
signals, conflicts with the positive drift matrix counters, or makes the
benchmark output less explicit.

## Expected Value

If the postulate survives, adaptive remote-free policy experiments can start
from a typed diagnostic action candidate instead of reinterpreting raw drift
counters at every call site.
