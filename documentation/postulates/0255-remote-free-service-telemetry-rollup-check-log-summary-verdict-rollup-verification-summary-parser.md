# Postulate 0255: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Parser

Date: 2026-07-03

## Claim

Saved verdict rollup verification summary JSON can be parsed back into typed
summary reports so dashboard archives can validate aggregate verifier-summary
artifacts.

## Rationale

Experiment 0262 added compact JSON summaries over saved verdict rollup
verification artifacts. Those aggregate artifacts are useful for release
dashboards, but the archive still needs a typed reload path that checks whether
flat counters and grouped coverage fields agree.

The parser should validate the summary schema, all flat counters,
`status_coverage`, and `drift_fields`. This catches stale or hand-edited
dashboard summaries even when they remain syntactically valid JSON.

## Test

Add public parsers for verdict rollup verification summary JSON lines and saved
logs.

Focused tests should prove:

- formatted verifier-summary JSON round-trips back into a typed summary;
- saved logs can contain human-readable lines around the summary JSON;
- schema drift is rejected;
- missing grouped fields are rejected;
- grouped status coverage drift is rejected;
- grouped drift-field coverage drift is rejected.

Real evidence should parse the verifier-summary log emitted by Experiment
0262.

## Expected Outcome

The postulate survives if the real verifier-summary JSON reloads as two
records, one matched artifact, one drifted artifact, and one `records` drift
bucket, while inconsistent grouped fields are rejected.
