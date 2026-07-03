# Postulate 0199: Remote-Free Runtime Window Collection

Date: 2026-07-03

## Claim

A service-window collection helper can gather runtime summaries from registered
owners through short mutable owner borrows, then route those summaries through
the shared service-window runner without changing guarded retune behavior.

## Rationale

Experiment 0206 added routed service-window observations, but callers still
had to borrow an owner runtime, collect its summary, release the borrow, and
then route the observation manually. A live service event loop needs this
borrow pattern expressed once so it can collect owner telemetry without
holding owner borrows while the coordinator applies, confirms, or rolls back
runtime state.

## Experiment

Add a collection helper that:

- accepts owner IDs for one service window;
- borrows each registered owner only while collecting its summary;
- routes the collected summary through `observe_service_window`;
- merges returned service-window stats;
- reports missing owner IDs before collection;
- reports collector errors with owner ID context;
- preserves the shared service mutation budget across owners.

Benchmark the helper with real runtime-collected owner windows that cover a
confirmed owner, a rolled-back owner, and a mutation-limited owner.

## Falsification

The postulate fails if the helper cannot express the borrow pattern without
unsafe code, if missing owner or collector errors lose owner ID context, if
mutation budget becomes owner-local, or if real allocation counters diverge
from the routed service-window benchmark.

## Expected Value

If the postulate survives, Locus will have a reusable service-loop primitive
for collecting and routing owner runtime telemetry.
