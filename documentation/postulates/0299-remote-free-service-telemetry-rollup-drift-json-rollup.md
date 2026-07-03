# Postulate 0299: Remote-Free Service Telemetry Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict JSON
records can aggregate into a dashboard rollup so release dashboards can track
matched and stale cohort rollup outcomes across runs.

## Rationale

Experiment 0306 proved that saved archive drift verdict rollup drift verdict
JSON reloads as typed matched and drifted reports. Release dashboards also
need a compact cohort view over many saved verdicts, rather than inspecting
each saved record one at a time.

The rollup should count total records, matched records, drifted records, and
the first drift field bucket. For the current matched plus stale cohort, it
should preserve the matched archive drift check and the stale `records=1`
archive drift check as one mixed dashboard cohort.

## Test

Combine the matched and stale JSON verdict logs saved by Experiment 0305 into
one persisted log. Run the validation example rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup <saved-verifier-summary-verification-rollup-verification-log.txt>
```

## Expected Outcome

The postulate survives if the combined saved verdict log emits a rollup with
`records=2`, `matched=1`, `drifted=1`, `drift_records=1`, and
`status_coverage` containing one matched record and one drifted record.
