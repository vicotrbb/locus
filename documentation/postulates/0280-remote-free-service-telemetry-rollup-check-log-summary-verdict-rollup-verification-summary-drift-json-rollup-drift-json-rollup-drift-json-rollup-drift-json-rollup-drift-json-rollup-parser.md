# Postulate 0280: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift verdict rollup
JSON can be parsed back into typed reports so dashboard archives can recheck
saved rollup-check verdict rollups.

## Rationale

Experiment 0287 proved that repeated-check rollup drift verdict rollup check
rollup drift verdict JSON records can aggregate into a dashboard rollup. That
rollup is useful as an archive artifact only if later release tooling can
reload it and preserve the same matched, drifted, and drift-field counters
without reparsing every source verdict record.

The parser is shared with earlier verifier-summary verification rollup JSON.
The next step is to prove that the repeated-check CLI path can reload the real
rollup artifact emitted by Experiment 0287.

## Test

Use the validation example parser mode against the real repeated-check rollup
drift verdict rollup check rollup drift verdict rollup JSON artifact from
Experiment 0287.

Focused tests should prove that the rollup JSON line and saved log parse back
into the original typed rollup, preserving `records=2`, `matched=1`,
`drifted=1`, and `drift_records=1`.

## Expected Outcome

The postulate survives if the real repeated-check rollup drift verdict rollup
check rollup drift verdict rollup JSON artifact reloads as a typed rollup with
`records=2`, `matched=1`, `drifted=1`, and `drift_records=1`.
