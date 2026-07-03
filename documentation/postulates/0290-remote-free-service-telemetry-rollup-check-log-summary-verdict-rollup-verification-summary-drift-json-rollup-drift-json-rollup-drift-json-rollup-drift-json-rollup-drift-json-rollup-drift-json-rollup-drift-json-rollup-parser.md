# Postulate 0290: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift verdict rollup
drift verdict rollup drift verdict rollup JSON can reload as a typed rollup so
dashboard archives can recheck saved verdict-rollup-check cohorts.

## Rationale

Experiment 0297 proved that saved repeated-check rollup drift verdict rollup
check rollup drift verdict rollup drift verdict rollup drift verdict JSON
records aggregate into a dashboard rollup. Long-lived dashboard archives need
that rollup JSON to reload as structured data later, preserving the matched,
drifted, and drift-field counters without relying on console text.

The parser should accept the saved compact JSON line for the dashboard rollup
schema and reconstruct the same typed summary counters.

## Test

Run the validation example parser mode against the real saved rollup artifact
from Experiment 0297.

## Expected Outcome

The postulate survives if the saved rollup artifact reloads as a typed rollup
with `records=2`, `matched=1`, `drifted=1`, and `drift_records=1`.
