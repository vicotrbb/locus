# Postulate 0258: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Parser

Date: 2026-07-03

## Claim

Verifier-summary drift verdict JSON can be parsed back into typed reports so
dashboard archives can recheck aggregate-summary verdict artifacts.

## Rationale

Experiment 0265 made verifier-summary drift checks emit compact verdict JSON.
The next archive-safety step is to reload that verdict JSON, reconstruct the
expected and actual verifier summaries, recompute the first drift, and reject
any stale or internally inconsistent verdict payload.

This closes the loop for dashboard archives that store aggregate-summary
verification as structured data instead of only text logs.

## Test

Add public line and log parsers for verifier-summary drift verdict JSON.

Focused tests should prove:

- matched verifier-summary verdict JSON parses back into the original typed
  report;
- drifted verifier-summary verdict JSON parses back with `field=records`;
- tampered `status` or `matched` fields are rejected;
- tampered `drift` payloads are rejected;
- nested expected or actual summary group drift is rejected before the verdict
  report is accepted.

Real evidence should parse the matched and controlled stale JSON artifacts
emitted by Experiment 0265.

## Expected Outcome

The postulate survives if real matched and stale verifier-summary verdict JSON
artifacts reload as typed reports with the same `matched` and `drifted`
statuses, and malformed verdict artifacts are rejected by focused tests.
