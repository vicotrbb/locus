# Postulate 0278: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift verdict JSON can
be parsed back into typed reports so dashboard archives can recheck saved
rollup-check verdict artifacts.

## Rationale

Experiment 0285 proved that repeated-check rollup drift verdict rollup check
rollup drift checks can emit compact verdict JSON for matched and drifted
outcomes. Those JSON artifacts are only useful for release archives if later
tooling can reload them as typed reports, recompute the first drift from the
nested expected and actual rollups, and reject inconsistent status or drift
payloads.

The parser is shared with earlier verifier-summary verification rollup
verification reports. The next step is to prove that the repeated-check CLI
path reloads the real matched artifact and the controlled stale `records=1`
artifact emitted by Experiment 0285.

## Test

Use the validation example parser mode against:

- the real matched repeated-check rollup drift verdict rollup check rollup
  drift verdict JSON artifact from Experiment 0285;
- the real controlled stale `records=1` verdict JSON artifact from Experiment
  0285.

Focused tests should prove that matched JSON reloads as the original typed
report, drifted JSON reloads with `field=records`, and the expected and actual
nested rollups survive the parse.

## Expected Outcome

The postulate survives if the real matched artifact reloads with
`status=matched` and the controlled stale artifact reloads with
`status=drifted`, `field=records`, expected `2`, and actual `1`.
