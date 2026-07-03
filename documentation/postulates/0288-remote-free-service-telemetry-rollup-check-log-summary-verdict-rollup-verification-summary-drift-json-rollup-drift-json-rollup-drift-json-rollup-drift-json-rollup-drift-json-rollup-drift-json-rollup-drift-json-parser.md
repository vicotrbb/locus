# Postulate 0288: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check rollup drift verdict rollup check rollup drift verdict
rollup drift verdict rollup drift verdict JSON can reload as typed reports so
dashboard archives can recheck matched and drifted verdict-rollup-check
outcomes.

## Rationale

Experiment 0295 proved that repeated-check rollup drift verdict rollup check
rollup drift verdict rollup drift verdict rollup drift checks can emit compact
JSON verdicts for matched and stale archives. Those verdict artifacts are
useful only if later tooling can reload them as typed reports and preserve both
the matched status and the first drift field.

The parser should find the schema-bearing JSON line inside a saved log with
surrounding console text and reconstruct the same typed report without rereading
the source verdict records or archived rollup.

## Test

Run the validation example parser mode against the real matched and controlled
stale JSON verdict logs from Experiment 0295.

## Expected Outcome

The postulate survives if the matched saved JSON verdict reloads as
`status=matched`, `records=2`, `matched=1`, `drifted=1`, and
`drift_records=1`, while the stale `records=1` verdict reloads as
`status=drifted` with `field=records`, expected `2`, and actual `1`.
