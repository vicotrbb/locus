# Postulate 0275: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Claim

Repeated-check rollup drift verdict rollup check rollup JSON can be parsed
back into typed reports so dashboard archives can recheck repeated verdict
rollup check rollups.

## Rationale

Experiment 0282 proved that repeated-check rollup drift verdict rollup check
JSON records aggregate into a dashboard rollup. That rollup is useful as an
archive artifact only if later release tooling can reload it and preserve the
same counters without reparsing every source verdict record.

The schema is intentionally shared with earlier verifier-summary verification
rollups. The next step is to prove that the repeated-check CLI path can reload
the real rollup artifact emitted by Experiment 0282.

## Test

Use the validation example parser mode against the real repeated-check rollup
drift verdict rollup check rollup JSON artifact from Experiment 0282.

Focused tests should prove:

- the rollup JSON line parses back into the original typed rollup;
- the rollup JSON log parser ignores non-JSON context lines;
- the parsed counters preserve `records=2`, `matched=1`, `drifted=1`, and
  `drift_records=1`;
- the real archived rollup reloads through the repeated-check CLI path.

## Expected Outcome

The postulate survives if the real repeated-check rollup drift verdict rollup
check rollup JSON artifact reloads as a typed rollup with `records=2`,
`matched=1`, `drifted=1`, and `drift_records=1`.
