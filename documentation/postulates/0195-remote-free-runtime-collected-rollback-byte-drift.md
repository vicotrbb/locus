# Postulate 0195: Remote-Free Runtime-Collected Rollback On Byte Drift

Date: 2026-07-03

## Claim

`RemoteFreeServiceRetuneGuard` can roll back a runtime-applied remote-free
candidate using only runtime-collected telemetry when the validation workload
reveals retained-byte drift after apply.

## Rationale

Experiments 0201 and 0202 proved runtime-collected apply, confirm, and
mutation-limit paths. Rollback still needs a real failed validation window.
Applying `drain_earlier` to the same workload cleans the drift, so a rollback
test needs a real workload change rather than a synthetic invalid summary.

A larger retained item size after apply is a plausible validation failure:
queue capacity and drain cadence may improve, while the retained-byte budget
derived from the previous workload shape becomes too small.

## Experiment

Add a guarded runtime benchmark sequence that:

- starts `RemoteFreeOwnerRuntime` with queue capacity 128, the queued-byte
  diagnostic config, and an initial empty drain policy;
- runs two real 4096-byte owner windows whose runtime-collected reports produce
  a stable `increase_queue_capacity_and_drain_earlier` apply decision;
- applies the candidate through `RemoteFreeServiceRetunePolicyApplicator`,
  installing queue capacity 256 and retaining rollback state;
- validates with a real owner window that allocates larger retained blocks and
  records their true byte size in runtime accounting;
- observes retained-byte drift from `RemoteFreeOwnerRuntime::drift_report`;
- rolls back through `RemoteFreeOwnerRuntime` at an empty boundary.

## Falsification

The postulate fails if the initial runtime-collected reports do not produce the
combined apply candidate, if the larger validation workload does not produce a
rollback decision from real runtime reports, if runtime rollback does not
restore queue capacity 128 and clear rollback state, or if submitted, drained,
released-byte, drain, or wait counters diverge from the measured windows.

## Expected Value

If the postulate survives, Locus will have runtime-collected evidence for all
guarded owner-runtime outcomes: apply, confirm, rollback, no-change, and
mutation limit.
