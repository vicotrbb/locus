# Postulate 0284: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check rollup drift verdict rollup check rollup drift verdict
rollup drift verdict JSON records can aggregate into a dashboard rollup so
release checks can summarize matched and drifted verdict-rollup-check outcomes.

## Rationale

Experiment 0291 proved that individual saved repeated-check rollup drift
verdict rollup check rollup drift verdict rollup drift verdict JSON artifacts
reload as typed reports. Release dashboards need a cohort view across many
saved verdict artifacts so they can count matched checks, drifted checks, and
the first drift field without inspecting each file by hand.

The rollup should reuse the parsed typed verdict records, not ad hoc text
matching, and should preserve the same matched, drifted, and drift-field
counters.

## Test

Concatenate the real matched and controlled stale JSON verdict logs from
Experiment 0290 into one saved log, then run the validation example rollup mode
against that combined log.

## Expected Outcome

The postulate survives if the combined saved verdict log aggregates into a
typed rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_records=1`, and the rollup JSON preserves the same status coverage and
drift-field counters.
