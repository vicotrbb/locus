# Postulate 0196: Remote-Free Runtime Retune Coordinator

Date: 2026-07-03

## Claim

A small reusable runtime retune coordinator can replace benchmark-local
guarded orchestration by owning the service guard and policy applicator, then
applying guard decisions to targeted `RemoteFreeOwnerRuntime` instances.

## Rationale

Experiments 0201 through 0203 proved runtime-collected apply, confirm,
rollback, no-change, and mutation-limit outcomes. The orchestration logic is
still repeated in benchmark modules: observe a summary, ask the guard for a
decision, translate apply decisions, and invoke runtime apply, confirm, or
rollback. That is the next barrier before live multi-owner runtime
orchestration.

## Experiment

Add a public coordinator that:

- owns one `RemoteFreeServiceRetuneGuard`;
- owns one `RemoteFreeServiceRetunePolicyApplicator`;
- observes a `RemoteFreeServiceRetuneSummary` for a targeted
  `RemoteFreeOwnerRuntime`;
- applies runtime no-change outcomes for hold and mutation-limit decisions;
- installs runtime configs for apply decisions;
- confirms runtime configs after clean validation windows;
- rolls back runtime configs after failed validation windows;
- preserves guard counters for service-wide mutation budget accounting.

Benchmark the coordinator with real runtime-collected owner windows covering
one confirmed owner, one rolled-back owner, and one mutation-limited owner.

## Falsification

The postulate fails if the coordinator hides guard decisions, loses runtime
operation outcomes, applies raw candidates without the typed applicator, lets
mutation budget become per-owner rather than service-wide, or diverges from the
real allocation counters measured by the benchmark-local orchestration.

## Expected Value

If the postulate survives, Locus will have a reusable API boundary for the
next multi-owner remote-free runtime orchestration experiments.
