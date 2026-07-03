# Postulate 0189: Remote-Free Guarded Policy Application

Date: 2026-07-03

## Claim

A production-facing remote-free retune application API should only translate
guarded `apply` decisions into validated queue and drain-policy configs. All
other guarded decisions should be observable no-change outcomes, and telemetry
should not be able to mutate live policy directly.

## Rationale

Experiments 0195 and 0196 measured the guard decision surface: apply, confirm,
rollback, and mutation-limit decisions. The next risk is API shape. If callers
consume raw candidates directly, service telemetry can become an implicit
mutation path. A narrow typed application layer should force candidate
application through validated `RemoteFreeQueuedByteDrainConfig` values and
checked capacity growth.

## Experiment

Add a small policy application planner that accepts:

- the current `RemoteFreeQueuedByteDrainConfig`;
- a `RemoteFreeServiceRetuneGuardDecision`;
- a queue-capacity growth factor greater than one.

The planner should:

- return no-change outcomes for collect, hold, confirm, rollback, and
  mutation-limit decisions;
- map `drain_earlier` apply decisions to the current queued-byte drain config;
- map `increase_queue_capacity` apply decisions to a checked larger queue
  capacity while preserving the retained pending-item window and byte budget;
- map `increase_queue_capacity_and_drain_earlier` apply decisions to the same
  checked larger queue capacity plus the queued-byte drain policy;
- reject non-actionable apply candidates and queue-capacity overflow.

Wire the planner into the guarded service benchmark so candidate validation
still runs through real `Vec<u8>` allocation and remote-free release paths.

## Falsification

The postulate fails if non-apply decisions can produce an applied config, if
non-actionable candidates are accepted as live policy changes, if capacity
growth overflows or silently saturates, or if the guarded service benchmark
changes the real submitted, drained, released-byte, or decision counters.

## Expected Value

If the postulate survives, Locus will have a narrow production-facing bridge
from measured guard decisions to validated policy configs while preserving the
separation between telemetry and live mutation.
