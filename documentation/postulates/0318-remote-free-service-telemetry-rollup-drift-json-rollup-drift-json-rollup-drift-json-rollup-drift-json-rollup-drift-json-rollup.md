# Postulate 0318: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict JSON records can aggregate into a dashboard rollup so release
dashboards can summarize stored aggregate rollup recheck cohort verification
outcomes.

## Rationale

Experiment 0325 proved that saved aggregate rollup verification JSON verdict
records reload as typed reports for matched and stale archive checks. Release
dashboards also need a compact cohort-level summary over those stored verdicts
so they can track how many saved aggregate rollup rechecks matched, drifted,
and drifted by first field.

The rollup must be computed from the saved JSON verdict records, not from
manually copied console text.

## Test

Combine the saved matched and stale JSON verdict records from Experiment 0325,
then run the dashboard rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the combined saved JSON verdict log produces a
dashboard rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_fields.records=1`.
