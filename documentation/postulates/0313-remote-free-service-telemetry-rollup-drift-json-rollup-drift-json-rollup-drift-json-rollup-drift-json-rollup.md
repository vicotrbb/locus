# Postulate 0313: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict JSON records
can aggregate into a dashboard rollup so release dashboards can summarize
stored rollup recheck cohort verification outcomes.

## Rationale

Experiment 0320 proved that saved matched and stale compact JSON verdict
records reload as typed reports. Release dashboards also need a compact
cohort-level summary so they can show how many stored rollup recheck outcomes
matched, drifted, and drifted by first field.

The rollup should be computed from the saved JSON records, not from manually
copied console text.

## Test

Combine the saved matched and stale JSON verdict records from Experiment 0320,
then run the dashboard rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the combined saved JSON verdict log produces a
dashboard rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_fields.records=1`.
