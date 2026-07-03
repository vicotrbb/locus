# Postulate 0321: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck JSON verdict records can reload as typed reports so
release dashboards can reprocess stored aggregate rollup recheck cohort
verification rollup recheck decisions.

## Rationale

Experiment 0328 proved that matched and controlled stale aggregate rollup
recheck cohort rollup recheck outcomes emit compact JSON verdicts. Release
dashboards need those saved verdict records to be reloadable later as typed
reports so archival decisions are not locked to console output.

The parser must preserve both the matched decision and the controlled stale
`records` drift decision.

## Test

Run the parser-only mode against the saved matched and stale JSON verdict
records from Experiment 0328:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched saved JSON verdict reloads as
`status=matched` with `records=2`, `matched=1`, `drifted=1`, and
`drift_records=1`, while the stale saved JSON verdict reloads as
`status=drifted` with `field=records`, expected `2`, and actual `1`.
