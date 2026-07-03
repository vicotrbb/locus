# Postulate 0245: Remote-Free Service Telemetry Rollup Check Log Summary JSON

Date: 2026-07-03

## Claim

Saved-log rollup check summaries can emit a compact JSON line with grouped
coverage fields while preserving the existing human-readable summary line.

## Rationale

The multi-record saved-log summary gives humans a compact job-level view of
record count, host coverage, and status coverage. Dashboard ingestion should
not parse the human token line by hand. A schema-tagged JSON line can carry the
same validated totals with grouped structure for typed consumers.

Compatibility still matters. The human-readable summary line should remain
the first line so existing release logs and simple token parsers continue to
work.

## Test

Add a JSON formatter for `RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary`.

Focused tests should prove:

- the human-readable summary line is unchanged;
- the JSON line is valid single-line JSON;
- flat fields match the typed summary;
- grouped `host_coverage` and `status_coverage` fields match the typed
  summary.

Real evidence should summarize the combined host-bearing and older
no-host-bundle saved log and record both the human and JSON summary lines.

## Expected Outcome

The postulate survives if the saved-log summary prints the existing human line
first and a schema-tagged JSON line second, with matching record, host
coverage, and status coverage totals.
