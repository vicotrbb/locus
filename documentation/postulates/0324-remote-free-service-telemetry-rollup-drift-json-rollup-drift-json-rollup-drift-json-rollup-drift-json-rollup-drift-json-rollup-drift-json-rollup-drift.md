# Postulate 0324: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup JSON can verify against saved source
verdict records so release dashboards can detect stale stored summary
outcomes.

## Rationale

Experiment 0331 proved that saved aggregate rollup recheck cohort rollup
recheck verdict rollup JSON reloads as a typed rollup. Release dashboards also
need to recompute that rollup from the saved source verdict records and reject
stale stored rollup artifacts.

The verifier must accept the real saved archive and reject a controlled stale
archive where only the archived top-level `records` count is changed to `1`.

## Test

Run the strict archive recheck mode against the saved source verdict records
and the saved rollup JSON artifact:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

Then run the report mode and strict mode against a controlled stale rollup
copy with top-level `records=1`:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-report <saved-json-verdict-log.txt> <stale-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-json-verdict-log.txt> <stale-rollup-log.txt>
```

## Expected Outcome

The postulate survives if the real archive verifies with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`, while the controlled stale
archive reports `field=records`, expected `2`, actual `1`, and the strict
mode exits nonzero with `CountDrift`.
