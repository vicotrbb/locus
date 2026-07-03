# Postulate 0267: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check rollup drift checks can emit compact verdict JSON so dashboard
archives can save matched and drifted repeated cohort-level rollup checks as
structured artifacts.

## Rationale

Experiment 0274 made archived repeated-check rollup JSON checkable against the
saved repeated-check verdict JSON records it summarizes. The strict mode is
useful for release gates, and the text report is useful while debugging, but
dashboard archives also need a machine-readable artifact for matched and
drifted repeated cohort-level checks.

The repeated-check drift report has the same expected rollup, actual rollup,
and first drift shape as the earlier verifier-summary rollup check report. A
single compact verdict JSON schema can therefore preserve the same typed
review surface while the CLI exposes it at the newer repeated-check mode.

## Test

Add a JSON-emitting validation example mode for repeated-check rollup drift
checks and focused tests that build reports through the repeated-check helper.

Focused tests should prove:

- a matched repeated-check rollup check emits `status=matched`,
  `matched=true`, and `drift=null`;
- a controlled stale `records=1` repeated-check rollup emits
  `status=drifted`, `matched=false`, and `drift.field=records`;
- expected and actual nested rollup summaries are preserved in the JSON;
- strict verification still rejects the stale repeated-check rollup with
  `CountDrift`.

Real evidence should emit matched and controlled stale `records=1` verdict JSON
from the saved Experiment 0273 and 0274 artifacts.

## Expected Outcome

The postulate survives if the real matched repeated-check rollup check emits
structured JSON with `drift=null` and the controlled stale `records=1` repeated
check emits structured JSON with expected `records=2`, actual `records=1`, and
`drift.field=records`.
