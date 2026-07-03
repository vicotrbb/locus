# Postulate 0303: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict JSON can reload as typed reports so release dashboards
can recheck stored matched and stale cohort rollup check outcomes.

## Rationale

Experiment 0310 proved that repeated-check dashboard archive drift verdict
rollup drift reports emit compact JSON verdicts. Release dashboards also need
to reload those persisted verifier JSON records later and recover the same
typed matched or drifted report without recomputing source logs.

The parser should recover the expected rollup, actual rollup, status, matched
flag, and first drift field from saved compact JSON lines.

## Test

Parse the matched and stale JSON verdict logs saved by Experiment 0310 through
the validation example parser mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-verifier-summary-verification-rollup-verification-log.txt>
```

## Expected Outcome

The postulate survives if the matched artifact reloads as `status=matched`
with `records=2`, `matched=1`, `drifted=1`, and `drift=null`, while the stale
artifact reloads as `status=drifted` with `field=records`, expected `2`, and
actual `1`.
