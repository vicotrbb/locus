# Postulate 0272: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup checks can emit compact verdict JSON
so dashboard archives can save matched and drifted repeated verdict rollup
checks as structured artifacts.

## Rationale

Experiment 0279 made archived repeated-check rollup drift verdict rollup JSON
checkable against the saved repeated-check rollup drift verdict JSON records it
summarizes. The strict check is useful for release gates, but dashboards also
need a structured artifact that records matched and drifted verdict rollup
check outcomes without scraping human text.

The check report has the same expected rollup, actual rollup, and first drift
shape as earlier rollup checks. The existing compact verdict JSON schema should
therefore preserve the same review surface while the CLI exposes it at this
newer repeated verdict rollup check layer.

## Test

Add a JSON-emitting validation example mode for repeated-check rollup drift
verdict rollup checks and focused tests that build reports through the archived
verdict rollup check helper.

Focused tests should prove:

- a matched repeated verdict rollup check emits `status=matched`,
  `matched=true`, and `drift=null`;
- a controlled stale `records=1` repeated verdict rollup emits
  `status=drifted`, `matched=false`, and `drift.field=records`;
- expected and actual nested rollup summaries are preserved in the JSON;
- strict verification still rejects the stale repeated verdict rollup with
  `CountDrift`.

Real evidence should emit matched and controlled stale `records=1` verdict JSON
from the saved Experiment 0277 and 0279 artifacts.

## Expected Outcome

The postulate survives if the real matched repeated verdict rollup check emits
structured JSON with `drift=null` and the controlled stale `records=1` repeated
verdict rollup check emits structured JSON with expected `records=2`, actual
`records=1`, and `drift.field=records`.
