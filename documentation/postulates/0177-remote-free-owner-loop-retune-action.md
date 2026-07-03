# Postulate 0177: Remote-Free Owner-Loop Retune Action

Date: 2026-07-03

## Claim

The queued-byte owner-loop example should log the same
`RemoteFreeQueuedByteRetuneAction` recommendation that the retune benchmarks
assert, without changing the owner-side release path or the measured queue
counters.

## Rationale

Experiment 0184 moved retune-action selection into
`RemoteFreeQueuedByteDriftReport`, but only benchmarks currently print and
assert the action. Runtime-facing example code should show how a service owner
loop can observe drift, log the recommendation, and still keep queue draining
and allocator-specific release explicit.

## Experiment

Update `remote_free_queued_byte_owner_loop` to build a drift report from
`RemoteFreeDrainController::status_for_queue` at each owner control point.
Print the final `retune_hint`, `retune_action`, over-target counters, and
queue-backpressure observation in the stable completion line.

Assert through the example output that the known-good queued-byte owner loop
still reports:

- `full_count=0`;
- `policy_drains=4`;
- max pending 64;
- max queued bytes 655,360;
- max wait 2 bursts;
- mean wait 1.500 bursts;
- `retune_action=keep_config`.

## Falsification

The postulate is weakened if logging `retune_action` changes queue counters,
requires benchmark-only helpers, hides allocator-specific release logic, or
reports any action other than `keep_config` for the known-good owner loop.

## Expected Value

If the postulate survives, application-facing code has a concrete pattern for
observing drift and logging a benchmark-first retune recommendation from the
same public API used by the validation benchmarks.
