# Postulate 0316: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
reports can emit compact JSON verdicts so release dashboards can preserve
typed aggregate rollup recheck cohort verification outcomes.

## Rationale

Experiment 0323 proved that saved aggregate rollup JSON verifies against saved
source verdict records, accepting the real archive and rejecting a controlled
stale `records=1` archive.

Release dashboards also need compact machine-readable verdicts for those
aggregate rollup rechecks. The matched and stale verdict JSON records should
reload later and preserve both the matched cohort counters and the stale
`records` drift payload.

## Test

Emit compact JSON verdicts for the matched and stale aggregate rollup
verification reports:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

Then reload both saved JSON verdict logs through the parser-only path:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched verdict emits and reloads as
`status=matched`, `matched=true`, and `drift=null`, while the stale verdict
emits and reloads as `status=drifted`, `matched=false`, and a `records` drift
with expected `2` and actual `1`.
