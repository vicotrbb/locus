# Postulate 0306: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift reports can emit compact JSON verdicts so
release dashboards can preserve typed rollup recheck outcomes.

## Rationale

Experiment 0313 proved that the saved dashboard rollup artifact reloads as a
typed rollup, matches the source verifier records, and rejects a controlled
stale rollup. Release dashboards also need to preserve the outcome of that
rollup recheck as compact JSON, not only as human console text.

The JSON verdict should preserve the matched and drifted statuses, expected
and actual rollup counters, and the first drift field when a saved rollup is
stale.

## Test

Run the validation example JSON verdict mode for the saved matched rollup and
the controlled stale rollup from Experiment 0313:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-verifier-summary-verification-log.txt> <saved-verifier-summary-verification-rollup-log.txt>
```

Then reload each saved JSON verdict:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-rollup-recheck-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched verdict emits and reloads as
`status=matched`, `matched=true`, and `drift=null`, while the stale verdict
emits and reloads as `status=drifted`, `matched=false`, and `field=records`
with expected `2` and actual `1`.
