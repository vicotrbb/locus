# Postulate 0262: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Verifier-summary drift verdict rollup checks can emit compact verdict JSON so
dashboard archives can save matched and drifted cohort-level checks as
structured artifacts.

## Rationale

Experiment 0269 made archived verifier-summary drift verdict rollup JSON
checkable against the saved verifier-summary drift verdict JSON records it
summarizes. The strict mode is useful for release gates, but dashboards also
need a structured artifact that records matched and drifted rollup-check
outcomes without scraping human text.

The JSON verdict should preserve expected and actual rollup summaries plus the
first drift field. That keeps stale cohort-level artifacts reviewable after the
original command has completed.

## Test

Add a compact JSON formatter and validation example mode for verifier-summary
drift verdict rollup check reports.

Focused tests should prove:

- a matched rollup check emits `status=matched`, `matched=true`, and
  `drift=null`;
- a controlled stale `records=1` archived rollup emits `status=drifted`,
  `matched=false`, and `drift.field=records`;
- expected and actual nested rollup summaries are preserved in the JSON;
- strict verification still rejects the stale rollup with `CountDrift`.

Real evidence should emit matched and controlled stale `records=1` verdict JSON
from the saved Experiment 0267 and 0269 artifacts.

## Expected Outcome

The postulate survives if the real matched rollup check emits structured JSON
with `drift=null` and the controlled stale `records=1` rollup check emits
structured JSON with expected `records=2`, actual `records=1`, and
`drift.field=records`.
