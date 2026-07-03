# Postulate 0259: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup

Date: 2026-07-03

## Claim

Verifier-summary drift verdict JSON records can be aggregated into a dashboard
rollup so repeated aggregate-summary checks have cohort-level status and drift
coverage.

## Rationale

Experiment 0266 made verifier-summary drift verdict JSON reloadable as typed
reports. A dashboard that runs this check repeatedly needs a compact cohort
view that counts matched artifacts, drifted artifacts, and the first drift
field for each drifted artifact.

This should make aggregate-summary verifier health visible without scanning
each individual verdict manually.

## Test

Add a public summarizer and JSON formatter for saved verifier-summary drift
verdict JSON logs.

Focused tests should prove:

- a mixed matched-plus-drifted verifier-summary verdict log rolls up to two
  records, one matched record, one drifted record, and one `records` drift
  bucket;
- the rollup JSON uses its own schema;
- grouped `status_coverage` and `drift_fields` counters match the flat fields;
- an empty log is rejected.

Real evidence should concatenate the matched and controlled stale artifacts
from Experiment 0265 and roll them up through the validation example.

## Expected Outcome

The postulate survives if the real mixed verifier-summary drift verdict log
rolls up to two records with one matched artifact, one drifted artifact, and
one `records` drift bucket, and the focused tests reject missing records.
