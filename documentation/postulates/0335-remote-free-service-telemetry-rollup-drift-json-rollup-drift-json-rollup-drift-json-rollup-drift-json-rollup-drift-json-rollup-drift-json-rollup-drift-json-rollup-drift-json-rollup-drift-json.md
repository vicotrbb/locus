# Postulate 0335: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift verdict rollup drift verdict rollup drift verdict rollup drift verdict
rollup recheck verdict rollup drift drift reports can emit compact JSON
verdicts so release dashboards can archive summary recheck decisions.

## Rationale

Experiment 0342 proved that the archived dashboard rollup can verify against
saved source verdict records and reject a controlled stale `records=1` rollup.
Release dashboards also need those matched and stale outcomes saved as compact
JSON verdict records so later tooling can archive and reload the exact summary
recheck decisions.

The compact verdict records must preserve both the matched rollup counters and
the stale `records` drift payload.

## Test

Run the JSON verdict mode and parser-only mode against the matched and stale
artifacts from Experiment 0342:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-output.txt>
```

## Expected Outcome

The postulate survives if the matched archive emits and reloads with
`status=matched`, `matched=true`, and `records=2`, while the controlled stale
archive emits and reloads with `status=drifted`, `matched=false`,
`field=records`, expected `2`, and actual `1`.
