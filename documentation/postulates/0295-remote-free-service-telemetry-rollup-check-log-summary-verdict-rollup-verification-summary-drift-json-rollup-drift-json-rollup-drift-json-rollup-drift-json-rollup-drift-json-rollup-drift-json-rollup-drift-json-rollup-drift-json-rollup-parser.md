# Postulate 0295: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Repeated-check dashboard archive drift verdict rollup JSON can reload as a
typed rollup so dashboard archives can recheck saved archive drift verdict
cohorts.

## Rationale

Experiment 0302 proved that saved repeated-check dashboard archive drift
verdict JSON records aggregate into a dashboard rollup. Long-lived release
dashboard archives need that rollup JSON to reload as structured data later,
preserving matched archive checks, stale archive checks, and drift-field
counters without relying on console text.

The parser should accept the saved compact JSON line for the rollup schema and
reconstruct the same typed summary counters.

## Test

Run the validation example parser mode against the real saved dashboard rollup
artifact from Experiment 0302.

## Expected Outcome

The postulate survives if the saved rollup artifact reloads as a typed rollup
with `records=2`, `matched=1`, `drifted=1`, and `drift_records=1`.
