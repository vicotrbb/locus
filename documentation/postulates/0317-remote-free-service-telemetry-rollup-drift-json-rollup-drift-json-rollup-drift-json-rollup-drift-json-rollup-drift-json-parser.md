# Postulate 0317: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict JSON records can reload as typed reports so release dashboards can
recheck stored aggregate rollup recheck cohort verification outcomes.

## Rationale

Experiment 0324 proved that aggregate rollup verification outcomes can be
emitted as compact JSON verdict records for both matched and stale archive
checks. Release dashboards need to reload those saved verdicts later without
depending on the original source records or console text.

The parser-only path should preserve the matched aggregate cohort counters and
the stale `records` drift payload exactly enough to recheck stored dashboard
records.

## Test

Copy the saved matched and stale compact JSON verdict records from Experiment
0324 into a fresh evidence directory, then reload each saved record through
the parser-only verifier:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

## Expected Outcome

The postulate survives if the matched saved JSON record reloads as a typed
`matched` report with two records, one matched check, one drifted check, and
one `records` drift bucket, while the stale saved JSON record reloads as a
typed `drifted` report with `field=records`, expected `2`, and actual `1`.
