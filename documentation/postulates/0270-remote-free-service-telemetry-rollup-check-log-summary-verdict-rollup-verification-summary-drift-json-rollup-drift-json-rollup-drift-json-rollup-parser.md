# Postulate 0270: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup JSON can be parsed back into typed
reports so dashboard archives can recheck repeated cohort-level verdict
rollups.

## Rationale

Experiment 0277 made repeated-check rollup drift verdict JSON records
aggregate into a dashboard rollup. That rollup is useful as a saved archive
artifact only if later release and dashboard tooling can reload it and validate
the grouped counters.

The rollup JSON intentionally uses the existing verifier-summary verification
rollup schema. The repeated-check CLI path should expose the parser directly,
and focused tests should prove the rollup generated from repeated-check drift
verdict records reloads as the same typed report.

## Test

Add a repeated-check parse mode alias to the validation example and focused
tests that build a repeated-check verdict rollup JSON line from matched and
stale repeated-check drift verdict records before parsing it back.

Focused tests should prove:

- repeated-check verdict rollup JSON parses back into the original typed
  rollup;
- grouped counters remain internally consistent after parse;
- the parsed rollup reports two records, one matched check, one drifted check,
  and one `records` drift bucket;
- real repeated-check verdict rollup JSON reloads through the CLI.

## Expected Outcome

The postulate survives if the real repeated-check verdict rollup artifact
reloads with `records=2`, `matched=1`, `drifted=1`, and
`drift_fields.records=1`.
