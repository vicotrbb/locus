# Postulate 0269: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict JSON records can be aggregated into a
dashboard rollup so repeated cohort-level check outcomes can be summarized.

## Rationale

Experiment 0276 proved that repeated-check rollup drift verdict JSON can be
saved and later reloaded as typed reports. A dashboard archive needs one more
cohort view: a rollup of many repeated-check verdict records showing how many
checks matched, how many drifted, and which first drift fields were observed.

The underlying schema is intentionally shared with the earlier verifier-summary
rollup check report, so the existing typed rollup shape should remain valid.
The repeated-check CLI path should expose that aggregation directly and tests
should prove the rollup is built from verdict JSON produced by the
repeated-check drift helper.

## Test

Add a repeated-check rollup mode alias to the validation example and focused
tests that build matched and stale verdict JSON through the repeated-check
drift helper before aggregating them.

Focused tests should prove:

- a mixed matched-plus-drifted repeated-check verdict JSON log rolls up to two
  records;
- the mixed log reports one matched check, one drifted check, and one
  `records` drift bucket;
- the rollup JSON uses the existing verifier-summary verification rollup schema
  and keeps grouped counters consistent;
- an empty repeated-check verdict log is rejected by the shared summarizer.

Real evidence should concatenate the matched and controlled stale
`records=1` repeated-check verdict JSON artifacts from Experiment 0275 and
emit a dashboard rollup.

## Expected Outcome

The postulate survives if the real mixed repeated-check verdict log emits a
dashboard rollup with `records=2`, `matched=1`, `drifted=1`, and
`drift_fields.records=1`.
