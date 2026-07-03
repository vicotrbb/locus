# Postulate 0197: Remote-Free Runtime Retune Owner Registry

Date: 2026-07-03

## Claim

A small owner registry around `RemoteFreeServiceRuntimeRetuneCoordinator` can
address multiple `RemoteFreeOwnerRuntime` instances by stable owner IDs while
preserving one shared service-level mutation budget.

## Rationale

Experiment 0204 added a reusable coordinator, but benchmark code still passed
owner runtime references manually. Live multi-owner orchestration needs a
clear API for registering owner runtimes, addressing them, and routing
runtime-collected summaries to the same coordinator state.

## Experiment

Add a reusable owner registry that:

- owns one `RemoteFreeServiceRuntimeRetuneCoordinator`;
- owns a collection of `RemoteFreeOwnerRuntime` instances;
- returns stable owner IDs when runtimes are registered;
- routes a summary for one owner through the shared coordinator;
- reports missing owner IDs explicitly;
- exposes owner runtime state for inspection after apply, confirm, rollback,
  or no-change outcomes.

Benchmark the registry with real runtime-collected owner windows that cover a
confirmed owner, a rolled-back owner, and a mutation-limited owner addressed by
owner ID.

## Falsification

The postulate fails if owner IDs address the wrong runtime, if mutation budget
is accidentally scoped per owner, if missing owner IDs are not reported, if
runtime outcomes diverge from coordinator decisions, or if real allocation
counters diverge from the existing coordinator benchmark.

## Expected Value

If the postulate survives, Locus will have the first reusable multi-owner
remote-free retune orchestration surface.
