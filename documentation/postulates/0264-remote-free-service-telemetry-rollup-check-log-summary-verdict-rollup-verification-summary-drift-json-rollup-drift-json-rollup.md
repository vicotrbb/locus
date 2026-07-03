# Postulate 0264: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Verifier-summary drift verdict rollup check JSON records can be aggregated into
a dashboard rollup so repeated cohort-level checks have status and drift
coverage.

## Rationale

Experiment 0271 made verifier-summary drift verdict rollup check JSON
reloadable as typed reports. The next dashboard step is to summarize many of
those check artifacts so repeated cohort-level checks can be reviewed without
opening every artifact.

The rollup should report total records, matched records, drifted records, and
first-drift buckets. That preserves the same evidence shape already used for
verifier-summary drift verdict cohorts.

## Test

Add a summarizer over saved verifier-summary drift verdict rollup check JSON
records and expose it through the validation example.

Focused tests should prove:

- a mixed matched-plus-drifted check JSON log rolls up to two records;
- the mixed log reports one matched check, one drifted check, and one
  `records` drift bucket;
- the rollup JSON uses the existing verifier-summary verification rollup
  schema and keeps grouped counters consistent;
- an empty log is rejected.

Real evidence should combine the matched and controlled stale artifacts saved
by Experiment 0270, then summarize them into one dashboard rollup.

## Expected Outcome

The postulate survives if the real combined log reports two records, one
matched artifact, one drifted artifact, and one `records` drift bucket.
