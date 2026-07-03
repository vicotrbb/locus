# Postulate 0304: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict JSON records can aggregate into a dashboard rollup so
release dashboards can track matched and stale cohort rollup check outcomes
across runs.

## Rationale

Experiment 0311 proved that saved matched and stale cohort rollup drift check
JSON verdicts reload as typed reports. Release dashboards also need a compact
cohort view over many saved verdicts, not just one stored verdict at a time.

The rollup should count total records, matched records, drifted records, and
the first drift field bucket across saved verifier JSON records.

## Test

Combine the matched and stale JSON verdict logs saved by Experiment 0310 into
one persisted log. Run the validation example rollup mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup <saved-verifier-summary-verification-rollup-verification-log.txt>
```

## Expected Outcome

The postulate survives if the combined saved verifier JSON log emits a rollup
with `records=2`, `matched=1`, `drifted=1`, `drift_records=1`, and status
coverage containing one matched record and one drifted record.
