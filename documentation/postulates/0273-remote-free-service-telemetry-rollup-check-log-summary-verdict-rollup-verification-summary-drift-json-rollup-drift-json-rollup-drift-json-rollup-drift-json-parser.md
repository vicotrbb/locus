# Postulate 0273: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check JSON can be parsed back into
typed reports so dashboard archives can recheck repeated verdict rollup check
artifacts.

## Rationale

Experiment 0280 made repeated-check rollup drift verdict rollup checks emit
compact verdict JSON. That JSON is useful as an archive artifact only if later
dashboard and release tooling can reload it, recompute the first drift from the
nested expected and actual summaries, and reject internally inconsistent
verdict fields.

The schema is intentionally shared with earlier rollup verification reports.
The next step is to prove that verdict JSON produced by the repeated verdict
rollup check helper reloads through the typed parser and is available through
the repeated-check CLI path.

## Test

Add a repeated-check parse mode alias to the validation example and focused
tests that build verdict JSON through the repeated verdict rollup check helper
before parsing it back.

Focused tests should prove:

- matched repeated verdict rollup check JSON parses back into the original
  typed report;
- drifted repeated verdict rollup check JSON parses back with `field=records`;
- expected and actual nested rollup summaries survive the parse;
- real matched and controlled stale `records=1` repeated verdict rollup check
  JSON artifacts reload through the CLI.

## Expected Outcome

The postulate survives if real matched and stale repeated verdict rollup check
JSON artifacts reload as typed reports, with the matched artifact reporting
`status=matched` and the stale artifact reporting `status=drifted` with
`field=records`.
