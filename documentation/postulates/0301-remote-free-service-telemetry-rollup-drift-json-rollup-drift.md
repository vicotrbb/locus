# Postulate 0301: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup JSON can verify against saved source verdict records so release
dashboards can reject stale cohort rollup artifacts.

## Rationale

Experiment 0308 proved that archived cohort rollup JSON reloads as a typed
rollup. Release dashboards also need a strict recheck path that recomputes the
rollup from saved matched and stale source verdict records and rejects a stale
archived rollup.

The verifier should compare typed counters, not console text, and report the
first drift field with expected and actual values.

## Test

Verify the real Experiment 0307 saved source verdict log against its archived
rollup JSON. Then mutate the archived rollup JSON to `records=1` and rerun
the report and strict verifier modes.

## Expected Outcome

The postulate survives if the real archived rollup verifies with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`, while the stale archive
reports `field=records`, expected `2`, actual `1`, and fails strict
verification with `CountDrift`.
