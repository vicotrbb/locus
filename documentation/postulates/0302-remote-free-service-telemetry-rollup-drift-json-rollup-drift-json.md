# Postulate 0302: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift reports can emit compact JSON verdicts so release dashboards can store
matched and stale cohort rollup check outcomes as machine readable records.

## Rationale

Experiment 0309 proved that archived cohort rollup JSON verifies against saved
source verdict records and rejects stale archives. Release dashboards also
need to persist the verification result itself as a compact JSON record, so a
matched archive and a stale archive can be stored and rechecked later without
parsing console-only text.

The JSON verdict should preserve the expected rollup, actual rollup, matched
flag, status string, and first drift field.

## Test

Run the compact JSON verifier mode against the real matched archived rollup
and the controlled stale `records=1` archived rollup from Experiment 0309.
Then parse both saved verifier JSON logs back through the validation example.

## Expected Outcome

The postulate survives if the real archive emits and reloads `status=matched`,
`matched=true`, and `drift=null`, while the stale archive emits and reloads
`status=drifted`, `matched=false`, `field=records`, expected `2`, and actual
`1`.
