# Postulate 0252: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Drift JSON

Date: 2026-07-03

## Claim

Verdict rollup drift verification can emit structured JSON verdicts so
dashboard jobs can archive matched and drifted rollup checks without parsing
stderr.

## Rationale

Experiment 0259 added strict drift verification for archived verdict rollup
JSON, but strict mode returns an error on drift. That is correct for release
gates, but dashboards also need a non-failing artifact that records expected
rollup counters, archived rollup counters, status, and the first drift field.

The JSON verdict should mirror the saved-log summary verification verdict
shape so downstream jobs can process both levels of dashboard validation with
the same model.

## Test

Add a public report builder and JSON formatter for verdict rollup drift
verification.

Focused tests should prove:

- a matched archived verdict rollup emits `status=matched`, `matched=true`,
  `drift=null`, and expected plus actual rollups;
- a stale archived `records` counter emits `status=drifted`, `matched=false`,
  and a `drift` object with `field=records`, expected `2`, and actual `1`;
- the strict verifier still rejects the same stale archived rollup.

Real evidence should emit JSON verdicts for the real mixed verdict rollup and
for a controlled stale `records=1` archived rollup.

## Expected Outcome

The postulate survives if dashboard jobs can archive structured matched and
drifted verdict rollup checks without relying on process failure text.
