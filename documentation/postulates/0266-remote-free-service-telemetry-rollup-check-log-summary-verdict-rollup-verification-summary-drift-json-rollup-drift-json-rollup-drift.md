# Postulate 0266: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived verifier-summary drift verdict rollup check rollup JSON can be checked
against the saved repeated-check verdict JSON records it summarizes so stale
repeated cohort-level rollups are rejected.

## Rationale

Experiment 0273 proved that repeated-check rollup JSON can reuse the existing
verifier-summary verification rollup parser. The next archive-safety step is
to recompute the rollup from saved repeated-check verdict JSON records and
compare it with the archived rollup artifact.

This catches dashboards that keep a stale repeated-check cohort artifact after
adding, removing, or editing individual repeated-check verdict records.

## Test

Add public check and strict verify helpers for repeated-check verdict rollup
JSON logs.

Focused tests should prove:

- a rollup recomputed from saved repeated-check verdict JSON records matches
  the archived rollup JSON;
- a controlled stale `records=1` archived rollup reports `records` drift;
- the strict verifier rejects that stale rollup with `CountDrift`;
- grouped rollup drift is still rejected by the parser before comparison.

Real evidence should check the mixed rollup emitted by Experiment 0272 against
the mixed repeated-check verdict JSON log it summarizes, then repeat with a
controlled stale `records=1` rollup.

## Expected Outcome

The postulate survives if the real mixed repeated-check rollup archive matches
the saved repeated-check verdict JSON records and a controlled stale
`records=1` rollup is rejected with expected `records=2` and actual
`records=1`.
