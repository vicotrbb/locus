# Postulate 0314: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup JSON can
reload as a typed rollup so release dashboards can recheck stored aggregate
rollup recheck cohort verification outcomes.

## Rationale

Experiment 0321 proved that saved matched and stale JSON verdict records can
aggregate into a compact dashboard rollup with one matched outcome, one
drifted outcome, and one `records` drift bucket.

Release dashboards also need the saved aggregate rollup artifact itself to
reload later without the original source verdict records. That parser-only
path should preserve the same aggregate counters from the saved JSON rollup.

## Test

Copy the saved aggregate JSON rollup from Experiment 0321 into a fresh
evidence directory, then reload it through the parser-only rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify <saved-rollup-log.txt>
```

## Expected Outcome

The postulate survives if the saved aggregate rollup reloads as a typed report
with `records=2`, `matched=1`, `drifted=1`, and `drift_fields.records=1`.
