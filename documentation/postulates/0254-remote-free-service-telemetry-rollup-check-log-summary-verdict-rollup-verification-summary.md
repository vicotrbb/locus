# Postulate 0254: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary

Date: 2026-07-03

## Claim

Saved verdict rollup verification JSON records can be aggregated into a compact
typed dashboard summary of matched and drifted rollup-verification artifacts.

## Rationale

Experiment 0261 made individual verdict rollup verification artifacts
self-checking. A dashboard archive also needs a small summary over many saved
verification artifacts so release notes can show how many archived verifier
artifacts still match and which drift counter fails first when they do not.

The summary should parse the same saved JSON lines, reject internally
inconsistent records through the existing parser, count matched and drifted
artifacts, and bucket the first verifier-rollup drift field. This keeps the
dashboard archive useful without reparsing human-readable status lines.

## Test

Add a public summarizer and formatter for saved verdict rollup verification
JSON logs.

Focused tests should prove:

- mixed matched and drifted verifier JSON records aggregate into two records;
- the summary buckets `records` drift from a stale verifier-rollup artifact;
- logs with no verifier JSON records are rejected;
- malformed verifier JSON records are rejected through the typed parser.

Real evidence should aggregate the matched and controlled stale `records=1`
verification logs emitted by Experiment 0261.

## Expected Outcome

The postulate survives if the real matched and drifted verifier JSON artifacts
aggregate into a typed summary with two records, one matched artifact, one
drifted artifact, and one `records` drift bucket.
