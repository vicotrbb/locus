# Postulate 0323: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup JSON can reload as a typed rollup so
release dashboards can recheck stored aggregate rollup recheck cohort
verification rollup recheck summary outcomes.

## Rationale

Experiment 0330 proved that saved matched and stale aggregate rollup recheck
cohort rollup recheck JSON verdict records aggregate into a compact dashboard
rollup. Release dashboards also need to reload that saved rollup artifact
later as a typed value, without depending on manually copied status text.

The typed reload must preserve the same counters from the saved rollup JSON
artifact.

## Test

Run the parser-only mode against the saved rollup JSON artifact from
Experiment 0330:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify <saved-rollup-log.txt>
```

## Expected Outcome

The postulate survives if the saved rollup JSON artifact reloads with
`records=2`, `matched=1`, `drifted=1`, and `drift_records=1`.
