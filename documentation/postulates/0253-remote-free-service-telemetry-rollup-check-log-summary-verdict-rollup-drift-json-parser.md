# Postulate 0253: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Verdict rollup verification JSON can be parsed back into typed reports so
dashboard archives can recheck their own drift-verdict artifacts.

## Rationale

Experiment 0260 added structured JSON verdicts for archived verdict rollup
drift checks. A dashboard can now save matched and drifted verification
artifacts without parsing stderr, but those saved artifacts also need a
self-check path.

The parser should validate schema, status, matched flag, expected rollup,
actual rollup, and drift payload consistency. This catches hand-edited or stale
dashboard verdict artifacts even when they remain valid JSON.

## Test

Add public parsers for verdict rollup verification JSON lines and saved logs.

Focused tests should prove:

- matched JSON verification verdicts round-trip back into typed reports;
- drifted JSON verification verdicts round-trip back into typed reports;
- status drift is rejected;
- drift payload mismatch is rejected;
- nested expected or actual rollup coverage drift is rejected.

Real evidence should parse the matched and controlled stale `records=1`
verification logs emitted by Experiment 0260.

## Expected Outcome

The postulate survives if archived verdict rollup verification JSON can be
reloaded as typed reports and malformed or inconsistent verdict artifacts are
rejected.
