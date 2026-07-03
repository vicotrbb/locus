# Postulate 0265: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Verifier-summary drift verdict rollup check rollup JSON can be parsed back
into typed reports so dashboard archives can recheck repeated cohort-level
check rollups.

## Rationale

Experiment 0272 aggregated saved verifier-summary drift verdict rollup check
JSON records into the existing verifier-summary verification rollup schema. If
that schema reuse is sound, the existing verifier-summary verification rollup
parser should reload these repeated-check rollup artifacts without introducing
another parallel parser or schema.

That keeps dashboard archive formats smaller and avoids duplicating counter
validation logic for the same records, matched, drifted, and drift-field
coverage shape.

## Test

Add a focused test proving that a rollup produced from saved verifier-summary
drift verdict rollup check JSON records is accepted by the existing
verifier-summary verification rollup JSON parser.

Real evidence should parse the rollup artifact saved by Experiment 0272.

## Expected Outcome

The postulate survives if the real repeated-check rollup artifact reloads with
two records, one matched check, one drifted check, and one `records` drift
bucket.
