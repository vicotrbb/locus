# Postulate 0320: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck outcomes can emit compact JSON verdicts so release
dashboards can archive aggregate rollup recheck cohort verification rollup
recheck decisions.

## Rationale

Experiment 0327 proved that saved aggregate rollup recheck cohort rollup JSON
reloads as a typed rollup and strict rechecks against saved source verdict
records. Release dashboards also need compact decision artifacts for those
rechecks so matched and drifted archive states can be stored, reloaded, and
aggregated later.

The verdict record must be emitted from the typed recheck report, not from
manually copied status text.

## Test

Run the compact JSON verdict mode against both the saved matched archive and a
controlled stale archive with `records=1`:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

Then reload the saved JSON verdicts through the parser mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched archive emits and reloads
`status=matched` with `drift=null`, while the controlled stale archive emits
and reloads `status=drifted` with `field=records`, expected `2`, and actual
`1`.
