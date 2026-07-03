# Postulate 0247: Remote-Free Service Telemetry Rollup Check Log Summary JSON Drift

Date: 2026-07-03

## Claim

Saved-log summary JSON verification can detect drift by comparing the archived
summary JSON line against a separately recomputed summary from the same saved
rollup-check log.

## Rationale

Experiment 0254 proved that an archived dashboard summary JSON line can be
parsed and checked for internal consistency. That does not prove the summary
still matches the saved CI log it claims to summarize.

A stronger verifier should independently recompute the summary from the saved
rollup-check JSON records, parse the archived dashboard summary JSON line, and
compare every typed counter. This catches truncation, stale dashboard records,
and manual edits that keep the JSON internally valid but no longer match the
source CI log.

## Test

Add a public verifier for a saved rollup-check log plus a saved summary log.

Focused tests should prove:

- a recomputed summary and archived summary JSON match for the same log;
- drift in record count is rejected;
- drift in host coverage is rejected;
- drift in status coverage is rejected;
- a missing archived summary JSON line is rejected.

Real evidence should regenerate the combined rollup-check log and saved
summary log, then verify that the archived summary matches the recomputed
summary from the same source log.

## Expected Outcome

The postulate survives if a valid archived dashboard summary is accepted and a
separately edited but internally consistent summary JSON line is rejected
against the recomputed source-log summary.
