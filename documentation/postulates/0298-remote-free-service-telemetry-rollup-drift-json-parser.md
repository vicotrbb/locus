# Postulate 0298: Remote-Free Service Telemetry Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict JSON
can reload as typed reports so release dashboards can recheck stored matched
and stale cohort rollup outcomes.

## Rationale

Experiment 0305 proved that repeated-check dashboard archive drift verdict
rollup drift reports emit compact JSON verdicts. Release dashboards also need
to load those saved JSON verdict records later and recover the same typed
matched or drifted report without recomputing from source logs.

The parser should reject schema drift and recover the same counters and drift
payload that the formatter emitted. A parser-only check over persisted matched
and stale artifacts proves the saved records are durable enough for dashboard
rechecks.

## Test

Parse the matched and stale JSON verdict logs saved by Experiment 0305 through
the validation example parser mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-verifier-summary-verification-rollup-verification-log.txt>
```

## Expected Outcome

The postulate survives if the matched artifact reloads as `status=matched`
with two records, one matched source check, one drifted source check, and
`drift=null`, while the stale artifact reloads as `status=drifted` with
`field=records`, expected `2`, and actual `1`.
