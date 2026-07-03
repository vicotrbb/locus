# Postulate 0296: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived repeated-check dashboard archive drift verdict rollup JSON can verify
against saved archive drift verdict records so release dashboards catch stale
archive drift verdict cohorts.

## Rationale

Experiment 0303 proved that repeated-check dashboard archive drift verdict
rollup JSON reloads as a typed rollup. Release dashboards also need a strict
recheck path that recomputes the rollup from saved archive drift verdict JSON
records and rejects stale archived rollups.

The verifier should compare typed counters, not console text, and report the
first drift field with expected and actual values.

## Test

Verify the real Experiment 0302 saved source-record log against its archived
rollup JSON. Then mutate the archived rollup JSON to `records=1` and rerun the
strict verifier.

## Expected Outcome

The postulate survives if the real archived rollup verifies with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`, while the stale archive fails
strict verification with `CountDrift` on `records`, expected `2` and actual
`1`.
