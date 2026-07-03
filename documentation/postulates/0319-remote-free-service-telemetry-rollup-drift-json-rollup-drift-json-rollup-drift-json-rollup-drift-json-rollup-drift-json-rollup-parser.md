# Postulate 0319: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup JSON can reload as a typed rollup so release dashboards can
recheck stored aggregate rollup recheck cohort verification rollup outcomes.

## Rationale

Experiment 0326 proved that saved matched and stale aggregate rollup recheck
JSON verdict records aggregate into a compact dashboard rollup. Release
dashboards also need to reload that saved rollup artifact later, without
depending on console text parsing or manual field copying.

The reload must use the compact JSON rollup schema and preserve the same
typed counters that the source verdict cohort produced.

## Test

Run the parser-only reload mode against the saved rollup JSON artifact from
Experiment 0326:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify <saved-rollup-log.txt>
```

Then run the strict archive recheck mode against the saved source verdict log
and the saved rollup JSON artifact:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

## Expected Outcome

The postulate survives if both modes report `records=2`, `matched=1`,
`drifted=1`, and `drift_records=1` for the saved aggregate rollup recheck
cohort rollup artifact.
