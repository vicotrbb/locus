# Postulate 0276: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived repeated-check rollup drift verdict rollup check rollup JSON can be
verified against saved repeated-check rollup drift verdict rollup check JSON
records so stale dashboard rollups are rejected.

## Rationale

Experiment 0283 proved that a saved repeated-check rollup drift verdict rollup
check rollup JSON artifact can reload as a typed rollup. Reloading is not
enough for release gating because a stale dashboard rollup could still parse
successfully while disagreeing with the saved source verdict records.

The verifier must recompute the expected rollup from the source JSON records,
compare it with the archived rollup JSON, accept the matched archive, and
reject a controlled stale archive before it can be used as dashboard evidence.

## Test

Use the validation example strict verify mode against:

- the real mixed repeated-check rollup drift verdict rollup check JSON record
  log from Experiment 0282;
- the real matched repeated-check rollup drift verdict rollup check rollup JSON
  artifact from Experiment 0282;
- a controlled stale copy of the rollup JSON with top-level `records` changed
  from `2` to `1`.

Focused tests should prove that the strict verifier accepts the matched rollup
and rejects the stale rollup with `CountDrift { field: "records", expected: 2,
actual: 1 }`.

## Expected Outcome

The postulate survives if the real matched archive verifies successfully and
the controlled stale `records=1` archive fails strict verification with a
nonzero exit status and `field=records`.
