# Postulate 0263: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Claim

Verifier-summary drift verdict rollup check JSON can be parsed back into typed
reports so dashboard archives can recheck cohort-level check artifacts.

## Rationale

Experiment 0270 made verifier-summary drift verdict rollup checks emit compact
JSON artifacts. Those artifacts should not be write-only: dashboards and
release tooling need to reload them later, verify the embedded expected and
actual rollup summaries, and reject artifacts whose `status`, `matched`, or
`drift` payload disagrees with the embedded counters.

## Test

Add line and log parsers for verifier-summary drift verdict rollup check JSON.

Focused tests should prove:

- matched check JSON parses back into the original typed report;
- controlled stale `records=1` check JSON parses back with `field=records`;
- tampered `status` fields are rejected;
- tampered `drift` payloads are rejected;
- nested expected or actual rollup summary group drift is rejected before the
  verdict report is accepted.

Real evidence should parse the matched and controlled stale artifacts saved by
Experiment 0270.

## Expected Outcome

The postulate survives if the real matched artifact reloads as
`status=matched`, the controlled stale artifact reloads as `status=drifted`
with `field=records`, and tampered verdict fields are rejected in focused
tests.
