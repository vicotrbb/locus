# Postulate 0300: Remote-Free Service Telemetry Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup JSON can reload as a typed rollup so release dashboards can recheck
stored cohort rollup artifacts.

## Rationale

Experiment 0307 proved that saved matched and stale archive drift verdict
records aggregate into a compact dashboard rollup. Release dashboards also
need to reload the archived rollup JSON later and recover the same typed
cohort counters without re-reading every source verdict record.

The parser should recover total records, matched records, drifted records,
status coverage, and the `records` drift bucket from the saved compact rollup
JSON line.

## Test

Parse the saved rollup artifact from Experiment 0307 through the validation
example parser mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify <saved-verifier-summary-verification-rollup-log.txt>
```

## Expected Outcome

The postulate survives if the saved rollup artifact reloads as `records=2`,
`matched=1`, `drifted=1`, `drift_records=1`, and status coverage containing
one matched record and one drifted record.
