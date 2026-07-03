# Postulate 0282: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup drift verdict rollup
drift checks can emit compact verdict JSON so dashboard release checks can save
matched and drifted verdict-rollup-check outcomes.

## Rationale

Experiment 0289 proved that archived repeated-check rollup drift verdict
rollup check rollup drift verdict rollup JSON can be verified against the
saved source verdict records and that stale counters are rejected. That strict
path protects release checks, but dashboards also need a compact machine-
readable artifact that records both successful checks and controlled drift
without relying on human console text.

The JSON verdict should preserve the recomputed expected rollup, the archived
actual rollup, a boolean match flag, a status string, and the first drift field
when the archived rollup is stale.

## Test

Run the validation example JSON mode against the real matched source and rollup
artifacts from Experiment 0287. Then mutate the archived rollup from
`records=2` to `records=1` and run the same JSON mode against the stale
artifact.

## Expected Outcome

The postulate survives if the matched artifact emits compact JSON with
`status=matched`, `matched=true`, and `drift=null`, while the stale
`records=1` artifact emits compact JSON with `status=drifted`,
`matched=false`, and `drift.field=records`.
