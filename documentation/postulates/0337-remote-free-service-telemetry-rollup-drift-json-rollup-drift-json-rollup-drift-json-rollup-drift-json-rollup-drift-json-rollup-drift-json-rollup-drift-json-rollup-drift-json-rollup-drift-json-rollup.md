# Postulate 0337: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift drift JSON verdict records can
aggregate into a dashboard rollup so release dashboards can summarize stored
summary recheck decisions.

## Rationale

Experiment 0344 proved that saved matched and stale archived dashboard rollup
recheck JSON verdict records reload as typed reports. Release dashboards also
need those saved verdict records to aggregate into a compact status summary
without the original source verdict log or rollup artifact.

The aggregate must count both saved verdict records and preserve the bucket for
the stale `records` drift decision.

## Test

Combine the saved matched and stale JSON verdict records from Experiment 0344,
then run the aggregate rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup <combined-saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the aggregate report contains `records=2`,
`matched=1`, `drifted=1`, `drift_records=1`, and JSON
`"drift_fields":{"records":1}` for the matched plus stale summary recheck
cohort.
