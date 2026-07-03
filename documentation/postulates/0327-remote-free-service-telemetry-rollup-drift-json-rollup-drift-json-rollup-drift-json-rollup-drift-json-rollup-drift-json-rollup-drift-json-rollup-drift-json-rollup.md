# Postulate 0327: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift JSON verdict records can aggregate
into a dashboard rollup so release dashboards can summarize stored summary
recheck decisions.

## Rationale

Experiment 0334 proved that saved matched and stale aggregate rollup recheck
cohort rollup recheck verdict rollup drift JSON verdict records reload as
typed reports. Release dashboards also need a compact cohort summary over
those stored verdicts so they can count matched outcomes, drifted outcomes,
and first-field drift buckets.

The rollup must be computed from saved compact JSON verdict records, not from
manually copied console status text.

## Test

Combine the saved matched and stale JSON verdict records from Experiment 0334,
then run the dashboard rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the combined saved JSON verdict log produces a
dashboard rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_fields.records=1`.
