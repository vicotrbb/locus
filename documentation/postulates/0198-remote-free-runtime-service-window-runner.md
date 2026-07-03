# Postulate 0198: Remote-Free Runtime Service Window Runner

Date: 2026-07-03

## Claim

A reusable service-window runner over registered remote-free owner runtimes can
collect routed owner summaries, aggregate drift and decision counters, and
preserve the same guarded apply, confirm, rollback, and mutation-limit behavior
as direct registry calls.

## Rationale

Experiment 0205 added stable owner IDs and a registry, but the benchmark still
counted service-window telemetry and runtime decisions locally. A production
orchestration loop needs one small API that accepts routed owner observations,
uses the shared service coordinator, and returns counters that callers can log
without reimplementing decision bookkeeping.

## Experiment

Add a service-window observation runner that:

- accepts routed `(owner_id, summary)` observations;
- routes each observation through `RemoteFreeServiceRuntimeRetuneOwners`;
- records observed reports, drift maxima, and queue backpressure;
- records guard decision counts and runtime outcome counts;
- attaches owner ID context to routing or retune errors;
- keeps the guard state shared across all registered owners.

Benchmark the runner with real runtime-collected owner windows that cover a
confirmed owner, a rolled-back owner, and a mutation-limited owner.

## Falsification

The postulate fails if the runner hides missing-owner context, if decision
counters diverge from the guarded runtime outcomes, if mutation budget becomes
owner-local, or if real allocation counters diverge from the owner registry
benchmark.

## Expected Value

If the postulate survives, Locus will have a reusable bridge between
runtime-collected owner telemetry and service-level retune logging.
