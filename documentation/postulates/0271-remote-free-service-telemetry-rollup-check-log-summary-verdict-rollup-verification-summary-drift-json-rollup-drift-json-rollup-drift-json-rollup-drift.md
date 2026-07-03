# Postulate 0271: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived repeated-check rollup drift verdict rollup JSON can be checked against
the saved repeated-check rollup drift verdict JSON records it summarizes so
stale repeated cohort-level verdict rollups are rejected.

## Rationale

Experiment 0278 proved that repeated-check rollup drift verdict rollup JSON can
be parsed back into typed reports. Parsing proves that a rollup artifact is
well-formed, but not that it still matches the saved verdict records it claims
to summarize.

The verifier should recompute the rollup from saved repeated-check rollup drift
verdict JSON records and compare it with the archived rollup JSON. That keeps
dashboard cohort rollups from silently drifting after a stale artifact is
copied or edited.

## Test

Expose a repeated-check CLI alias for the existing typed rollup verifier and
run it against the saved real matched and controlled stale artifacts.

Focused evidence should prove:

- a rollup recomputed from saved repeated-check rollup drift verdict JSON
  records matches the archived rollup JSON;
- a controlled stale `records=1` archived rollup reports `records` drift;
- the strict verifier rejects that stale rollup with `CountDrift`;
- grouped rollup drift is still rejected by the parser before comparison.

## Expected Outcome

The postulate survives if the real mixed repeated-check verdict rollup matches
the saved verdict records and a controlled stale `records=1` rollup is rejected
with expected `records=2`, actual `records=1`, and `field=records`.
