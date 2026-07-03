# Postulate 0181: Remote-Free Service Retune Telemetry

Date: 2026-07-03

## Claim

Before Locus mutates remote-free policy adaptively, it should expose a
service-level telemetry summary that aggregates queued-byte drift reports from
multiple owner loops without changing queue capacity, drain cadence, or
queued-byte budgets.

## Rationale

Experiments 0184 through 0188 validated per-owner `retune_action` semantics.
An inference service will usually run many owner loops at once: scheduler
workers, KV-cache owners, and request-affine arenas can drift independently.
Adaptive policy needs a service view that can answer whether drift is isolated
to one owner or common across the service.

The first service-level step should remain observational. It should count
diagnostic actions and retain maxima for pending-item drift, queued-byte drift,
and queue backpressure before any runtime policy mutation is attempted.

## Experiment

Add a `RemoteFreeServiceRetuneSummary` that observes
`RemoteFreeQueuedByteDriftReport` values and reports:

- number of drift reports observed;
- number of reports needing retune;
- maximum pending items over target;
- maximum queued bytes over budget;
- number of reports with queue backpressure;
- counts for each `RemoteFreeQueuedByteRetuneAction`.

Validate the summary with focused tests and with a Criterion benchmark that
runs real `Vec<u8>` remote-free owner loops:

- a fixed queued-byte policy service where all owners remain clean;
- a service with one end-drain owner and remaining fixed-policy owners, where
  telemetry should report `drain_earlier` for the drifting owner.

## Falsification

The postulate fails if the summary changes owner-loop behavior, requires
duplicating allocator release logic outside the owner drain closure, cannot
distinguish isolated owner drift from service-wide clean policy, or adds enough
hot-path overhead to make telemetry unsuitable for benchmarked owner-loop
observability.

## Expected Value

If the postulate survives, adaptive remote-free policy work can use a measured
service-level observation source instead of interpreting individual owner
reports ad hoc.
