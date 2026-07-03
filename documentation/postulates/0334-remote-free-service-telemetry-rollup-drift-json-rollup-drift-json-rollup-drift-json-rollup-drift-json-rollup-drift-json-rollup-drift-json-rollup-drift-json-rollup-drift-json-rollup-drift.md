# Postulate 0334: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift JSON can verify against saved
source verdict records so release dashboards can reject stale stored summary
recheck rollups.

## Rationale

Experiment 0341 proved that a saved dashboard rollup artifact reloads as a
typed rollup. Release dashboards also need to compare that saved rollup against
the saved source verdict records so an archived but stale aggregate cannot be
accepted silently.

The verifier must accept the real saved rollup and reject a controlled stale
rollup where the top-level `records` count is changed from `2` to `1`.

## Test

Run the strict and report verifier modes against the saved source verdict log
and saved dashboard rollup from Experiments 0340 and 0341:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-json-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-report <saved-json-verdict-log.txt> <stale-rollup-log.txt>
```

## Expected Outcome

The postulate survives if the real saved rollup verifies with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`, while the controlled stale
rollup reports `field=records`, expected `2`, actual `1`, and strict mode exits
nonzero with `CountDrift`.
