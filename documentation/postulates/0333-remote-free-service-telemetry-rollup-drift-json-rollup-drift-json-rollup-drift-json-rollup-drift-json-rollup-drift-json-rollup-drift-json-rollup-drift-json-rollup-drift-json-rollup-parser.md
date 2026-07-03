# Postulate 0333: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift JSON can reload as a typed rollup
so release dashboards can recheck stored summary recheck outcomes.

## Rationale

Experiment 0340 proved that saved matched and stale JSON verdict records
aggregate into a compact dashboard rollup. Release dashboards need that saved
rollup artifact to remain reloadable later without re-reading the source JSON
verdict records, preserving the same status counters and drift buckets.

The parser must preserve the two-record aggregate, one matched decision, one
drifted decision, and the `records` drift bucket.

## Test

Run the parser-only rollup mode against the saved aggregate rollup JSON artifact
from Experiment 0340:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify <saved-aggregate-rollup-json-log.txt>
```

## Expected Outcome

The postulate survives if the saved aggregate rollup JSON reloads as a typed
rollup with `records=2`, `matched=1`, `drifted=1`, `drift_records=1`,
`drift_matched=0`, `drift_drifted=0`, and `drift_drift_records=0`.
