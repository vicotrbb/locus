# Postulate 0194: Remote-Free Runtime-Collected Multi-Owner Mutation Limit

Date: 2026-07-03

## Claim

A service-wide `RemoteFreeServiceRetuneGuard` can enforce its mutation limit
across multiple `RemoteFreeOwnerRuntime` instances using only
runtime-collected drift reports, while preserving real allocation and release
counters for every owner window.

## Rationale

Experiment 0201 proved the runtime-collected guarded apply-confirm path for
one owner runtime. The next service-level question is whether the same
runtime-collected telemetry can drive multiple owner retunes while keeping a
single guard mutation budget. This is closer to live orchestration than a
single-owner benchmark because owners can drift independently while the guard
still enforces one service-wide mutation limit.

## Experiment

Add a multi-owner guarded runtime benchmark that:

- keeps one `RemoteFreeServiceRetuneGuard` for the service;
- runs three separate `RemoteFreeOwnerRuntime` instances with real allocation
  windows;
- starts each owner with a queued-byte diagnostic config and an initial empty
  drain policy so runtime-collected drift is visible;
- feeds summaries collected from `RemoteFreeOwnerRuntime::drift_report` into
  the service guard;
- applies and confirms the first two stable owner candidates through the typed
  applicator and owner runtime;
- treats the third stable owner candidate as `mutation_limit_reached` and a
  runtime no-change outcome.

## Falsification

The postulate fails if runtime-collected reports do not produce two apply
decisions followed by one mutation-limit decision, if runtime install and
confirm counters diverge from guard decisions, if any owner retains stale
rollback state after confirmation, or if submitted, drained, released-byte,
drain, or wait counters diverge from the measured owner windows.

## Expected Value

If the postulate survives, Locus will have measured evidence that a
service-wide guard can coordinate multiple runtime-collected owner telemetry
streams and stop further live retunes when the mutation budget is exhausted.
