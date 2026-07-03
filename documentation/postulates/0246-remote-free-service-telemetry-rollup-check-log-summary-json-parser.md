# Postulate 0246: Remote-Free Service Telemetry Rollup Check Log Summary JSON Parser

Date: 2026-07-03

## Claim

Saved-log summary JSON lines can be parsed back into typed
`RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary` reports to
verify dashboard artifacts after CI logs are archived.

## Rationale

The saved-log summary JSON line is intended for dashboards. Once dashboards
store that line separately from the original CI log, Locus needs a way to
validate the stored artifact itself: schema, field types, flat totals, and
grouped coverage totals should all agree.

This parser should not rescan the original release-check records. It should
prove that a saved dashboard record is well typed, schema-tagged, and
internally consistent with the same typed summary used to produce it.

## Test

Add a public parser for saved-log summary JSON lines.

Focused tests should prove:

- formatting then parsing a summary round-trips to the same typed summary;
- schema drift is rejected;
- missing grouped fields are rejected;
- grouped host coverage drift is rejected;
- grouped status coverage drift is rejected.

Real evidence should write the real combined-log summary output and parse the
JSON summary line back into the same human-readable summary.

## Expected Outcome

The postulate survives if saved-log summary JSON lines can reconstruct typed
summaries while rejecting malformed, schema-drifted, or internally
inconsistent dashboard records.
