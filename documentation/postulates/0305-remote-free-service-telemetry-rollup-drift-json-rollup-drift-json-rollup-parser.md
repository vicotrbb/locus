# Postulate 0305: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup JSON can reload as a typed rollup so release
dashboards can recheck stored cohort rollup artifacts.

## Rationale

Experiment 0312 proved that saved matched and stale rollup drift verdict JSON
records aggregate into a compact dashboard rollup. Release dashboards also
need to reload that saved rollup later, compare it with the source verdict
records, and reject stale rollup artifacts.

The parser and verifier should preserve total records, matched records,
drifted records, and first drift field buckets from the saved rollup JSON.

## Test

Reload the saved rollup JSON emitted by Experiment 0312 through the validation
example parser mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify <saved-verifier-summary-verification-rollup-log.txt>
```

Then recheck the saved rollup against the combined source verifier JSON records:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-verifier-summary-verification-log.txt> <saved-verifier-summary-verification-rollup-log.txt>
```

Finally create a controlled stale `records=1` rollup and confirm the strict
recheck fails with a `records` count drift.

## Expected Outcome

The postulate survives if the saved rollup reloads with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`, the matched strict recheck
passes, and the controlled stale rollup fails strict verification with
expected `records=2` and actual `records=1`.
