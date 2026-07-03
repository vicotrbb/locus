# Postulate 0244: Remote-Free Service Telemetry Rollup Check Log Summary

Date: 2026-07-03

## Claim

Saved CI logs containing multiple rollup release-check JSON records can be
parsed into a typed summary of host coverage and status coverage.

## Rationale

A release job may validate more than one evidence root. Single-record parsing
proves that one JSON line is schema-tagged and internally consistent, but a CI
dashboard needs a compact job-level answer: how many records were present, how
many carried rollup host metadata, how many bundle rows carried capture host
metadata, and whether any failed bundle statuses appeared.

The summary should not reread rollup artifacts. It should treat already saved
JSON lines as evidence records, parse each one through the same typed parser,
and aggregate only values that were present in those records.

## Test

Add a multi-record saved-log parser and summary.

Focused tests should prove:

- a log with two JSON records reports two records;
- host coverage fields are summed across records;
- status coverage fields are summed across records;
- a log without rollup-check JSON records is rejected;
- malformed or schema-drifted records are rejected.

Real evidence should combine the host-bearing and older no-host-bundle saved
logs, then parse the combined log and record the summary line.

## Expected Outcome

The postulate survives if multi-record saved logs produce a stable typed
summary while preserving single-record parsing behavior and rejecting missing
or invalid evidence records.
