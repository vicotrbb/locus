# Postulate 0311: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift reports can emit
compact JSON verdicts so release dashboards can preserve typed rollup recheck
cohort verification outcomes.

## Rationale

Experiment 0318 proved that archived rollup recheck cohort rollup JSON can be
verified against saved source verdict records, accepting the real archive and
rejecting a controlled stale `records=1` archive.

Release dashboards also need compact machine-readable verdicts for those
checks. A human report is useful during triage, but stored dashboards should
preserve a typed `matched` or `drifted` verdict and enough drift payload to
recheck stale archive detection later.

## Test

Emit compact JSON verdicts for the matched and stale rollup recheck cohort
verification reports:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-rollup-recheck-verdict-log.txt> <saved-rollup-log.txt>
```

Then reload both saved JSON verdict logs through the parser-only path:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched verdict emits and reloads as
`status=matched`, `matched=true`, and `drift=null`, while the stale verdict
emits and reloads as `status=drifted`, `matched=false`, and a `records` drift
with expected `2` and actual `1`.
