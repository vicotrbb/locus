# Postulate 0307: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict JSON records can reload as typed
reports so release dashboards can recheck stored rollup recheck outcomes.

## Rationale

Experiment 0314 proved that matched and stale dashboard rollup recheck
outcomes emit compact JSON verdicts. Release dashboards also need to reload
those saved verdict artifacts later and preserve the matched or drifted
semantics without recomputing from source logs.

The parser should preserve the matched status, the stale `records` drift, and
the expected and actual rollup counters from the saved JSON verdict records.

## Test

Reload the matched and stale JSON verdict logs saved by Experiment 0314:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-rollup-recheck-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched artifact reloads as
`status=matched`, `matched=true`, and `drift=null`, while the stale artifact
reloads as `status=drifted`, `matched=false`, and `field=records` with
expected `2` and actual `1`.
