# Postulate 0139: Remote-Free Drain Tracker

Date: 2026-07-03

## Claim

The pending-age and queued-byte accounting used by the mixed-size remote-free benchmark should become a reusable owner-side helper instead of benchmark-local bookkeeping.

## Rationale

Experiment 0146 proved that `RemoteFreeDrainPolicy` can express the measured max-wait-2 policy, but the benchmark still owns the pending metadata needed to build observations. Future owner loops will need the same signals:

- pending item count;
- retained queued bytes;
- oldest pending age.

The helper should remain outside `RemoteFreeQueue` internals. Queue internals should stay focused on bounded handoff and owner draining, while runtime loops explicitly record submitted and drained work.

## Experiment

Add a `RemoteFreeDrainTracker` to `locus-alloc` that:

- records submitted work by logical turn and queued byte size;
- coalesces adjacent submissions from the same logical turn;
- records FIFO owner drains and subtracts released bytes;
- builds `RemoteFreeDrainObservation` for `RemoteFreeDrainPolicy`;
- reports invalid drain attempts instead of saturating silently.

Then wire `remote_free_mixed_size_policy` to use the helper instead of benchmark-local pending metadata.

## Falsification

The postulate is weakened if the helper makes the benchmark counters drift from experiment 0146, requires `RemoteFreeQueue` internals to change, or cannot detect basic invalid drain accounting.

## Expected Value

If the postulate survives, Locus will have a reusable owner-loop accounting layer that can feed drain policy decisions without coupling policy state to the queue primitive.
