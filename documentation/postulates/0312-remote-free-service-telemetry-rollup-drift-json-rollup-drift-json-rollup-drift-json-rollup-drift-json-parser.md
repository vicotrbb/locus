# Postulate 0312: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict JSON records
can reload as typed reports so release dashboards can recheck stored rollup
recheck cohort verification outcomes.

## Rationale

Experiment 0319 proved that matched and stale rollup recheck cohort archive
verification outcomes can be emitted as compact JSON verdict records. Durable
release evidence also needs the inverse path: saved JSON verdict records must
reload without their original console context and reproduce typed reports.

The parser-only path should preserve the matched cohort counters and stale
`records` drift payload exactly enough for release dashboards to recheck stored
outcomes later.

## Test

Copy the saved matched and stale compact JSON verdict records from Experiment
0319 into a fresh evidence directory, then reload each saved record through
the parser-only verifier:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched saved JSON record reloads as a typed
`matched` report with two records, one matched check, one drifted check, and
one `records` drift bucket, while the stale saved JSON record reloads as a
typed `drifted` report with `field=records`, expected `2`, and actual `1`.
