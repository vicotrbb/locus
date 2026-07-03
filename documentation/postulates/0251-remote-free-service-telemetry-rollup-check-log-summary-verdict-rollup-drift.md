# Postulate 0251: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Drift

Date: 2026-07-03

## Claim

Archived verdict rollup JSON can be compared against a recomputed rollup from
the same saved verdict log, catching stale dashboard rollups after publication.

## Rationale

Experiment 0258 proved that a saved verdict rollup JSON line can be parsed
back into a typed rollup and checked for internal consistency. The next risk is
external drift: a dashboard may publish an internally valid rollup that no
longer matches the verdict log it claims to summarize.

The verifier should recompute the verdict rollup from the saved verdict JSON
records, parse the archived verdict rollup JSON, and compare every counter in a
stable order.

## Test

Add a public verifier for saved verdict logs and archived verdict rollup JSON.

Focused tests should prove:

- a freshly generated archived verdict rollup matches the recomputed source log;
- `records` drift is rejected;
- status coverage drift is rejected through the archived rollup parser;
- drift-field coverage drift is rejected through the archived rollup parser.

Real evidence should verify the real mixed verdict log against the saved
verdict rollup output from Experiment 0258.

## Expected Outcome

The postulate survives if the verifier accepts the real archived verdict rollup
and rejects stale or internally inconsistent archived rollup JSON.
