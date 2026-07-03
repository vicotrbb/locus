# Postulate 0256: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift

Date: 2026-07-03

## Claim

Archived verifier-summary JSON can be checked against the saved verifier JSON
records it claims to summarize, so dashboard archives can detect stale
aggregate verifier summaries.

## Rationale

Experiment 0263 made verifier-summary JSON reloadable as typed reports. The
next archive-safety step is to recompute the verifier summary from the saved
verifier JSON records and compare that expected summary with the archived
summary artifact.

This catches dashboards that keep a stale aggregate summary after adding,
removing, or editing individual verifier artifacts.

## Test

Add public check and strict verify helpers for verifier-summary JSON logs.

Focused tests should prove:

- a summary recomputed from saved verifier JSON records matches the archived
  verifier-summary JSON;
- a controlled stale `records=1` archived summary reports `records` drift;
- the strict verifier rejects that stale summary with `CountDrift`;
- grouped summary drift is still rejected by the summary parser before
  comparison.

Real evidence should check the verifier-summary log emitted by Experiment 0262
against the combined verifier JSON log it summarizes.

## Expected Outcome

The postulate survives if the real verifier-summary archive matches the saved
verifier JSON records and a controlled stale `records=1` summary is rejected
with expected `records=2` and actual `records=1`.
