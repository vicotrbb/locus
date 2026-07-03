# Postulate 0173: Remote-Free Capacity Retune Action

Date: 2026-07-03

## Claim

The `increase_queue_capacity` retune hint should be tested as a capacity
action candidate against both queue backpressure and owner-side release
latency before it informs adaptive remote-free policy.

## Rationale

Experiment 0180 added a typed hint that maps queue backpressure to
`increase_queue_capacity`. That mapping is only a diagnostic first response.
Increasing queue capacity may remove producer backpressure while allowing more
remote-free work to sit pending until the owner drains it.

A useful action test must therefore report `full_count`, forced drains, max
pending items, max wait, and mean wait together.

## Experiment

Add a focused remote-free capacity retune benchmark with real `Vec`
allocations, `RemoteFreeQueue`, `RemoteFreeDrainController`, and
`RemoteFreeQueuedByteDriftReport`.

Measure three cases with the same 256-block, 8-burst workload and batch size
64:

- baseline capacity 64;
- candidate capacity 128;
- candidate capacity 256.

Assert the expected counters so the benchmark fails if the action hides a
latency tradeoff.

## Falsification

The postulate is weakened if increasing capacity removes `full_count` without
making wait growth visible, if the benchmark avoids real allocations, if the
retune hint conflicts with observed backpressure, or if expected counters are
not asserted.

## Expected Value

If the postulate survives, future adaptive-policy experiments can treat
capacity increases as measured action candidates rather than automatic policy
changes.
