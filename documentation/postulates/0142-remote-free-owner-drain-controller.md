# Postulate 0142: Remote-Free Owner Drain Controller

Date: 2026-07-03

## Claim

The owner-side policy wiring around `RemoteFreeQueue`, `RemoteFreeDrainTracker`, and `RemoteFreeDrainPolicy` should become a reusable controller instead of remaining benchmark-local glue.

## Rationale

Experiments 0147, 0148, and 0149 validated the same control pattern across mixed-size buffers, KV block handles, and request scratch arena returns:

- record successful remote-free submissions with a logical turn and retained byte size;
- compare tracker observations against queue pending counts;
- ask `RemoteFreeDrainPolicy` whether the owner should drain;
- record FIFO drain accounting when the domain allocator releases an item.

Keeping this wiring in every benchmark makes it easier for future runtime code to drift from the measured policy behavior. The reusable helper should remain outside `RemoteFreeQueue` internals so the queue stays focused on bounded handoff and owner draining.

## Experiment

Add a `RemoteFreeDrainController` to `locus-alloc` that:

- owns a `RemoteFreeDrainPolicy` and `RemoteFreeDrainTracker`;
- records submitted and drained work;
- builds policy status from a `RemoteFreeQueue` and current logical turn;
- rejects queue and tracker pending-count drift;
- exposes the policy decision without hiding domain-specific release logic.

Wire one real domain benchmark through the controller. The request scratch remote-free policy benchmark is the best first target because it has the smallest item count while still exercising real arena open, allocation, remote completion, queue drain, and arena close behavior.

## Falsification

The postulate is weakened if the controller forces unsafe release semantics, hides item-specific allocator behavior, changes the request benchmark counters, or cannot detect queue and tracker pending-count drift.

## Expected Value

If the postulate survives, Locus will have a reusable owner-side policy controller that keeps domain allocators explicit while reducing duplicated benchmark-local policy wiring.
