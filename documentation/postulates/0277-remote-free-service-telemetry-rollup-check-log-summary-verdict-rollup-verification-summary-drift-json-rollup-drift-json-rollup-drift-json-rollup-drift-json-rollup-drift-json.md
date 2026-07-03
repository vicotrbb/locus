# Postulate 0277: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift checks can emit
compact verdict JSON so dashboard release checks can save matched and drifted
rollup-check outcomes.

## Rationale

Experiment 0284 proved that archived repeated-check rollup drift verdict
rollup check rollup JSON can be verified against saved source verdict records.
The strict verifier is useful for pass or fail gating, but dashboard archives
also need compact JSON verdict artifacts that preserve matched checks, drifted
checks, nested expected and actual rollups, and the first drift field.

The schema is shared with earlier verifier-summary verification rollup
verification reports. The next step is to prove that the repeated-check CLI
path emits compact JSON for both a real matched archive and a controlled stale
`records=1` archive.

## Test

Use the validation example JSON verdict mode against:

- the real mixed repeated-check rollup drift verdict rollup check JSON record
  log from Experiment 0282;
- the real matched repeated-check rollup drift verdict rollup check rollup JSON
  artifact from Experiment 0282;
- the controlled stale `records=1` rollup artifact from Experiment 0284.

Focused tests should prove that the JSON formatter emits a matched artifact
with `drift=null` and a drifted artifact with `field=records`, expected `2`,
and actual `1`.

## Expected Outcome

The postulate survives if the real matched check emits `status=matched` with
`drift=null`, and the controlled stale check emits `status=drifted` with
`drift.field=records`, expected `2`, and actual `1`.
