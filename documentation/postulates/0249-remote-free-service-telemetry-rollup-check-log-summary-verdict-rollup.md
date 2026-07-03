# Postulate 0249: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup

Date: 2026-07-03

## Claim

Saved-log summary verification verdict JSON records can be aggregated across
multiple CI logs into a compact dashboard rollup.

## Rationale

Experiment 0256 added one verdict JSON record for a matched or drifted
archived summary. Dashboards also need a cohort view across many saved CI logs:
how many verdicts were ingested, how many matched, how many drifted, and which
counter fields drifted.

The aggregation should parse the same verdict JSON schema emitted by the
verifier, reject malformed records, and produce a compact JSON rollup that can
be archived beside dashboard inputs.

## Test

Add a public parser and summarizer for saved-log summary verification verdict
JSON records.

Focused tests should prove:

- matched verdict JSON parses back into the typed report;
- drifted verdict JSON parses back into the typed report;
- a mixed verdict log summarizes matched and drifted counts;
- drift fields are counted by known summary counter;
- the rollup JSON is single-line and schema-tagged.

Real evidence should combine the real matched verdict and controlled
`records=1` drifted verdict, then emit a rollup with one matched verdict, one
drifted verdict, and one `records` drift.

## Expected Outcome

The postulate survives if multiple saved verdict JSON records can produce a
typed and machine-readable dashboard rollup while rejecting malformed verdicts.
