# Postulate 0243: Remote-Free Service Telemetry Rollup Check JSON Parser

Date: 2026-07-03

## Claim

Rollup release-check JSON lines can be parsed back into the typed
`RemoteFreeServiceTelemetryCollectionSummaryRollupCheck` report so saved CI
logs can be rechecked without rereading the rollup artifact.

## Rationale

Release logs may outlive the artifact workspace that produced them. The JSON
line now carries the same validated values as the release-check report, plus
grouped copies for typed consumers. A parser lets downstream tools treat saved
logs as evidence records and validate that the flat fields and grouped fields
agree.

The parser should not recompute artifact integrity because it does not read
the artifact. It should instead prove that a saved JSON line is schema-tagged,
well typed, internally consistent, and able to reconstruct the same report
that produced it.

## Test

Add a public parser for successful release-check JSON lines.

Focused tests should prove:

- formatting then parsing a check report round-trips to the same typed report;
- grouped fields are checked against flat fields;
- schema drift is rejected;
- missing or wrongly typed fields are rejected;
- count drift between flat fields and grouped fields is rejected.

Real evidence should parse the JSON line emitted for the host-bearing rollup
and the older no-host-bundle rollup.

## Expected Outcome

The postulate survives if saved release-check JSON lines can reconstruct typed
rollup-check reports while rejecting malformed, schema-drifted, or internally
inconsistent records.
