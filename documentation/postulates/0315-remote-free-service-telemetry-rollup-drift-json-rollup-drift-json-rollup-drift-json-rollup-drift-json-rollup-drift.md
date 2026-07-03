# Postulate 0315: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup JSON can
verify against saved source verdict records so release dashboards can reject
stale aggregate rollup recheck cohort verification outcomes.

## Rationale

Experiment 0322 proved that the saved aggregate rollup artifact reloads as a
typed rollup with two records, one matched outcome, one drifted outcome, and
one `records` drift bucket.

Release dashboards also need to compare that saved aggregate rollup back to
its saved source JSON verdict records. A matching archive should pass strict
verification, while a stale archive with changed aggregate counts should fail
with an explicit count drift.

## Test

Verify the saved aggregate rollup from Experiment 0321 against the saved
source JSON verdict records from the same cohort:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

Then create a controlled stale aggregate rollup with `records=1` and confirm
strict verification fails with a `records` count drift while report mode
preserves the drift payload.

## Expected Outcome

The postulate survives if the real saved aggregate rollup verifies
successfully against the saved source JSON verdict records, while the
controlled stale `records=1` aggregate rollup fails strict verification with
expected `2` and actual `1`.
