# Postulate 0268: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict JSON can be parsed back into typed reports
so dashboard archives can recheck repeated cohort-level check artifacts.

## Rationale

Experiment 0275 made repeated-check rollup drift checks emit compact verdict
JSON. That JSON is useful as an archive artifact only if later dashboard and
release tooling can reload it, recompute the first drift from the nested
expected and actual summaries, and reject stale or internally inconsistent
verdict fields.

The schema is intentionally shared with the earlier verifier-summary rollup
check report. The next step is to prove that verdict JSON produced by the
repeated-check drift helper reloads through the typed parser and is available
through the repeated-check CLI path.

## Test

Add a repeated-check parse mode alias to the validation example and focused
tests that build verdict JSON through the repeated-check drift helper before
parsing it back.

Focused tests should prove:

- matched repeated-check verdict JSON parses back into the original typed
  report;
- drifted repeated-check verdict JSON parses back with `field=records`;
- parser validation still recomputes status and drift from expected and actual
  nested rollups;
- real matched and controlled stale `records=1` repeated-check verdict JSON
  artifacts reload through the CLI.

## Expected Outcome

The postulate survives if real matched and stale repeated-check rollup drift
verdict JSON artifacts reload as typed reports, with the matched artifact
reporting `status=matched` and the stale artifact reporting `status=drifted`
with `field=records`.
