# Postulate 0340: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift verdict rollup drift verdict rollup drift verdict rollup drift verdict
rollup recheck verdict rollup drift drift drift reports can emit compact JSON
verdicts so release dashboards can archive summary recheck decisions.

## Rationale

Experiment 0347 proved that archived dashboard rollup JSON can verify against
saved source verdict records and reject stale stored summary recheck rollups.
Release dashboards also need compact machine-readable verdicts for those
recheck outcomes so they can archive and reprocess the decision itself.

The JSON verdict path must preserve both sides of the check: the real saved
rollup should emit `status=matched`, while a controlled stale rollup with
top-level `records=1` should emit `status=drifted` with a `records` drift.

## Test

Run JSON verdict mode and parser-only mode against the saved source verdict log
plus matched and stale dashboard rollup artifacts from Experiment 0347:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-output.txt>
```

## Expected Outcome

The postulate survives if the matched archive emits and reloads
`status=matched`, `matched=true`, and `records=2`, while the stale archive
emits and reloads `status=drifted`, `matched=false`, `field=records`,
expected `2`, and actual `1`.
