# Postulate 0309: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup JSON can reload as a typed
rollup so release dashboards can recheck stored rollup recheck cohort
artifacts.

## Rationale

Experiment 0316 proved that saved matched and stale rollup recheck verdict
JSON records aggregate into a compact dashboard rollup. Release dashboards
also need to reload that saved rollup artifact later and preserve the mixed
cohort counters without recomputing from source verdict records.

The parser should preserve total records, matched records, drifted records,
and first drift field buckets from the saved rollup JSON.

## Test

Reload the saved rollup JSON emitted by Experiment 0316 through the validation
example parser mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify <saved-verifier-summary-verification-rollup-log.txt>
```

## Expected Outcome

The postulate survives if the saved rollup reloads with `records=2`,
`matched=1`, `drifted=1`, `drift_records=1`, and status coverage containing
one matched record and one drifted record.
