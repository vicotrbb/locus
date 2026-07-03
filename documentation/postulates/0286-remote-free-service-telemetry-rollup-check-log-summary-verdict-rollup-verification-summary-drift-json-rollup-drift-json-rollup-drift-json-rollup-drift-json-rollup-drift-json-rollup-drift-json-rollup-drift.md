# Postulate 0286: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Claim

Archived repeated-check rollup drift verdict rollup check rollup drift verdict
rollup drift verdict rollup JSON can be verified against saved
verdict-rollup-check JSON records so stale dashboard verdict rollups are
rejected.

## Rationale

Experiment 0293 proved that repeated-check rollup drift verdict rollup check
rollup drift verdict rollup drift verdict rollup JSON can reload as a typed
rollup. A parser protects schema readability, but it does not prove that a
saved rollup still matches the source verdict records it summarizes.

The next integrity boundary is a strict verifier that recomputes the expected
rollup from saved source verdict records and rejects an archived rollup when
any persisted counter has drifted.

## Test

Use the validation example strict verifier against the real repeated-check
rollup drift verdict rollup check rollup drift verdict rollup drift verdict
source JSON records and their archived rollup JSON artifact from Experiment
0292.

Then mutate the archived rollup from `records=2` to `records=1` and run both
report mode and strict mode against the same source records.

## Expected Outcome

The postulate survives if the real archived rollup verifies with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`, while the stale `records=1`
rollup reports `field=records`, expected `2`, actual `1`, and fails strict
verification with `CountDrift`.
