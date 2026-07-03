# Postulate 0279: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift verdict JSON
records can be aggregated into a dashboard rollup so saved rollup-check
verdict outcomes can be summarized.

## Rationale

Experiment 0286 proved that repeated-check rollup drift verdict rollup check
rollup drift verdict JSON can reload as typed reports. Individual verdict
artifacts are useful for archive rechecks, but dashboard release views need a
cohort summary that counts matched checks, drifted checks, and drift fields
across saved rollup-check verdict artifacts.

The summarizer is shared with earlier verifier-summary verification rollup
verification verdict records. The next step is to prove that the repeated-check
CLI path can aggregate a real mixed log containing a matched JSON verdict and a
controlled stale `records=1` JSON verdict.

## Test

Use the validation example rollup mode against a mixed log containing:

- the real matched repeated-check rollup drift verdict rollup check rollup
  drift verdict JSON artifact from Experiment 0285;
- the real controlled stale `records=1` verdict JSON artifact from Experiment
  0285.

Focused tests should prove that matched verdict JSON records count as matched,
drifted verdict JSON records count as drifted, and drift fields are preserved
in the rollup.

## Expected Outcome

The postulate survives if the real mixed log reports `records=2`, `matched=1`,
`drifted=1`, and `drift_fields.records=1` through the repeated-check CLI
rollup path.
