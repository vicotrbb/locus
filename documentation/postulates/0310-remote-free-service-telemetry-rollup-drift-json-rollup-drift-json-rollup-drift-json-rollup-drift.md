# Postulate 0310: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup JSON can verify against
saved source verdict records so release dashboards can reject stale rollup
recheck cohort artifacts.

## Rationale

Experiment 0317 proved that the saved rollup recheck cohort rollup JSON
reloads as a typed rollup. Release dashboards also need to compare a saved
rollup artifact back to its source verdict records, accepting matching
archives and rejecting stale archives.

The strict verifier should recompute the rollup from saved source records and
reject any drift in total records, status coverage, or first drift field
buckets.

## Test

Verify the saved rollup from Experiment 0317 against the saved source verdict
records from Experiment 0316:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-rollup-recheck-verdict-log.txt> <saved-rollup-log.txt>
```

Then create a controlled stale rollup with `records=1` and confirm strict
verification fails with a `records` count drift while report mode preserves
the drift payload.

## Expected Outcome

The postulate survives if the real saved rollup verifies successfully against
the source verdict records, while the controlled stale `records=1` rollup
fails strict verification with expected `2` and actual `1`.
