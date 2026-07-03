# Postulate 0159: Remote-Free Queued-Byte Owner Loop Example

Date: 2026-07-03

## Claim

A small owner-loop example can show how to use the queued-byte remote-free
policy candidate without hiding allocator-specific release logic.

## Rationale

Experiment 0166 showed that `RemoteFreeDrainPolicy::with_max_queued_bytes` can
match the retained-byte and release-latency counters of the current max-wait-2
policy on the mixed-size trace. The policy is now measured, but runtime users
still need a concrete pattern for choosing a byte budget and placing the owner
drain control point.

The example should not add a new abstraction yet. It should keep the important
boundary visible:

- domain code chooses the retained-byte budget;
- producers submit remote-free work to `RemoteFreeQueue`;
- the owner calls `RemoteFreeDrainController` at control points;
- allocator-specific release happens in the `drain_batch` closure.

## Experiment

Add a `locus-alloc` example that:

- derives a queued-byte budget from request concurrency, blocks per request,
  and representative block bytes;
- configures `RemoteFreeDrainPolicy::with_max_queued_bytes`;
- submits real `Vec` allocations through `RemoteFreeQueue`;
- records submits and drains through `RemoteFreeDrainController`;
- prints summary counters for retained bytes, drain rounds, and release wait.

## Falsification

The postulate is weakened if the example requires queue internals, hides the
release closure, fails under `cargo run`, or gives counters that do not match
the measured queued-byte policy behavior.

## Expected Value

If the postulate survives, the queued-byte policy becomes easier to evaluate as
a runtime integration candidate while keeping the current explicit ownership
boundary intact.
