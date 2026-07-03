# Postulate 0260: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Verifier-summary drift verdict rollup JSON can be parsed back into typed
reports so dashboard archives can recheck cohort-level aggregate-summary
verdict artifacts.

## Rationale

Experiment 0267 added a dashboard rollup for saved verifier-summary drift
verdict JSON records. The rollup is useful as an archive artifact only if a
later release check can reload it, validate its grouped counters, and recover
the same typed cohort counts.

This should catch archived rollup JSON with stale schema, missing grouped
fields, or flat and grouped counter drift.

## Test

Add public line and log parsers for verifier-summary drift verdict rollup JSON.

Focused tests should prove:

- a mixed matched-plus-drifted rollup JSON line parses back into the original
  typed rollup;
- a saved log containing human text plus rollup JSON reloads as the same
  typed rollup;
- grouped `status_coverage` drift is rejected;
- grouped `drift_fields` drift is rejected;
- logs without rollup JSON are rejected.

Real evidence should parse the rollup artifact emitted by Experiment 0267.

## Expected Outcome

The postulate survives if the real mixed verifier-summary drift verdict rollup
artifact reloads as two records with one matched artifact, one drifted
artifact, and one `records` drift bucket, while focused tests reject malformed
grouped counters and missing records.
