# Postulate 0285: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift verdict rollup
drift verdict rollup JSON can reload as a typed rollup so dashboard archives
can recheck saved verdict-rollup-check cohorts.

## Rationale

Experiment 0292 proved that saved repeated-check rollup drift verdict rollup
check rollup drift verdict rollup drift verdict JSON records aggregate into a
dashboard rollup. That rollup is useful as an archive artifact only if later
release tooling can reload it and preserve the same matched, drifted, and
drift-field counters without reparsing every source verdict record.

The parser should find the schema-bearing rollup JSON line inside a saved log
with surrounding console text and reconstruct the typed rollup.

## Test

Run the validation example parser mode against the real repeated-check rollup
drift verdict rollup check rollup drift verdict rollup drift verdict rollup
JSON artifact from Experiment 0292.

## Expected Outcome

The postulate survives if the real repeated-check rollup drift verdict rollup
check rollup drift verdict rollup drift verdict rollup JSON artifact reloads as
a typed rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_records=1`.
