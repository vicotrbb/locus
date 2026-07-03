# Postulate 0257: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON

Date: 2026-07-03

## Claim

Verifier-summary drift checks can emit compact verdict JSON so dashboard
archives can save matched and drifted aggregate-summary checks as structured
artifacts.

## Rationale

Experiment 0264 added typed drift checks for archived verifier-summary JSON.
The human report is useful for logs, but dashboards need a stable JSON verdict
artifact that preserves the recomputed summary, archived summary, status, and
first drift field.

This artifact should let later tooling store and parse aggregate-summary drift
checks without replaying raw text.

## Test

Add a compact JSON formatter and CLI mode for verifier-summary verification
reports.

Focused tests should prove:

- a matched verifier-summary check emits `status=matched`, `matched=true`, and
  `drift=null`;
- a controlled stale `records=1` summary emits `status=drifted`,
  `matched=false`, and `drift.field=records`;
- expected and actual nested summary counters remain visible in the verdict
  JSON;
- the strict verifier still rejects stale summaries with `CountDrift`.

Real evidence should run the new CLI mode against the archived Experiment 0262
verifier-summary log and against a controlled stale copy.

## Expected Outcome

The postulate survives if the real archive emits a matched JSON verdict and the
controlled stale archive emits a drifted JSON verdict with expected
`records=2` and actual `records=1`.
