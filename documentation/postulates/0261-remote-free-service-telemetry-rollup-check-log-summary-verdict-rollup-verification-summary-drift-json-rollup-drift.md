# Postulate 0261: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived verifier-summary drift verdict rollup JSON can be checked against the
saved verifier-summary drift verdict JSON records it summarizes so stale
cohort-level artifacts are rejected.

## Rationale

Experiment 0268 made verifier-summary drift verdict rollup JSON reloadable as a
typed cohort report. The next archive-safety step is to recompute the rollup
from saved verifier-summary drift verdict JSON records and compare it with the
archived rollup artifact.

This catches dashboards that keep a stale cohort-level artifact after adding,
removing, or editing individual verifier-summary drift verdict records.

## Test

Add public check and strict verify helpers for verifier-summary drift verdict
rollup JSON logs.

Focused tests should prove:

- a rollup recomputed from saved verifier-summary drift verdict JSON records
  matches the archived rollup JSON;
- a controlled stale `records=1` archived rollup reports `records` drift;
- the strict verifier rejects that stale rollup with `CountDrift`;
- grouped rollup drift is still rejected by the parser before comparison.

Real evidence should check the mixed rollup emitted by Experiment 0267 against
the mixed verifier-summary drift verdict JSON log it summarizes, then repeat
with a controlled stale `records=1` rollup.

## Expected Outcome

The postulate survives if the real mixed rollup archive matches the saved
verifier-summary drift verdict JSON records and a controlled stale `records=1`
rollup is rejected with expected `records=2` and actual `records=1`.
