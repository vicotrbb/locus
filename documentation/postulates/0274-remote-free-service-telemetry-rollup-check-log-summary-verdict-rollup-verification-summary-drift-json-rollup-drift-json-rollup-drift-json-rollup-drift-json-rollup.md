# Postulate 0274: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check JSON records can be
aggregated into a dashboard rollup so repeated verdict rollup check outcomes
can be summarized.

## Rationale

Experiment 0281 proved that repeated-check rollup drift verdict rollup check
JSON artifacts reload as typed reports. A single parsed artifact is useful for
archive rechecks, but dashboard and release tooling also need a cohort view
that counts matched checks, drifted checks, and drift fields across repeated
archives.

The schema is intentionally shared with earlier verifier-summary verification
rollups. The next step is to prove that the repeated-check CLI path can scan a
mixed saved log of matched and stale verdict JSON records and produce the same
typed rollup counters.

## Test

Use the validation example rollup mode against a real mixed log containing:

- the matched repeated verdict rollup check JSON artifact from Experiment 0280;
- the controlled stale `records=1` repeated verdict rollup check JSON artifact
  from Experiment 0280.

Focused tests should prove the aggregation path by building matched and
drifted JSON through the repeated verdict rollup check helper before
summarizing those records.

## Expected Outcome

The postulate survives if the mixed real log reports `records=2`, `matched=1`,
`drifted=1`, and `drift_fields.records=1` through the repeated-check CLI path.
