# Postulate 0294: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict JSON records can aggregate
into a dashboard rollup so release checks can summarize matched and stale
archive drift outcomes.

## Rationale

Experiment 0301 proved that saved repeated-check dashboard archive drift
verdict JSON reloads as typed reports for matched and controlled stale
archive outcomes. Release dashboards need a cohort rollup across many saved
archive drift verdict artifacts so they can count matched archive checks,
stale archive checks, and first drift fields without reading each artifact by
hand.

The rollup should reuse the parsed typed verdict records and preserve the same
status coverage and drift-field counters.

## Test

Concatenate the real matched and controlled stale archive drift verdict JSON
logs from Experiment 0300, then run the validation example rollup mode against
that combined log.

## Expected Outcome

The postulate survives if the combined saved verdict log aggregates into a
typed rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_records=1`.
