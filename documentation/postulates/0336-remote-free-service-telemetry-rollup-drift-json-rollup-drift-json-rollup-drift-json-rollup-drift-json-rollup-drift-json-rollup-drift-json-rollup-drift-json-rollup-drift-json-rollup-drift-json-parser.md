# Postulate 0336: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift drift JSON verdict records can
reload as typed reports so release dashboards can reprocess stored summary
recheck decisions.

## Rationale

Experiment 0343 proved that matched and controlled stale archived dashboard
rollup recheck outcomes emit compact JSON verdict records. Release dashboards
need those saved verdict records to be reloadable later without the source
verdict log or rollup artifact, preserving both the matched aggregate counters
and the stale `records` drift payload.

The parser must keep the matched aggregate counters intact and keep the stale
drift payload intact.

## Test

Run the parser-only mode against the saved matched and stale JSON verdict
records from Experiment 0343:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched saved JSON verdict reloads as
`status=matched` with `records=2`, `matched=1`, `drifted=1`, and
`drift_records=1`, while the stale saved JSON verdict reloads as
`status=drifted` with `field=records`, expected `2`, and actual `1`.
