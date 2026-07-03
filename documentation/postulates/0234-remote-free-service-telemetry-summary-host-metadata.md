# Postulate 0234: Remote-Free Service Telemetry Summary Host Metadata

Date: 2026-07-03

## Claim

Remote-free service telemetry collection summaries can record direct capture
host metadata while staying compatible with existing schema v1 validation.

## Rationale

Experiment 0241 added host metadata to regenerated rollup artifacts. That is
useful for the refresh process, but it does not preserve the original capture
host if the rollup is rebuilt elsewhere. Recording a small optional host object
inside `collection-summary.json` lets each evidence bundle carry the capture
context next to its copied outputs, manifest, validation summary, and artifact
byte counts.

The field should stay optional so previously archived schema v1 summaries keep
parsing and validating. The validator should parse the field when present, but
artifact integrity should remain tied to listed files and byte counts.

## Test

Add optional host metadata to parsed collection summaries and to collector
output. The collector should write:

- Rust target operating system;
- Rust target CPU architecture;
- hostname when the process exposes `HOSTNAME` or `COMPUTERNAME`.

Focused tests should prove both older summaries without host metadata and new
summaries with host metadata parse correctly. A real direct capture should
write a `collection-summary.json` with host metadata and still pass summary
validation.

## Expected Outcome

The postulate survives if existing summary validation remains compatible, the
collector writes a metadata-bearing direct-capture summary, and the validator
accepts that summary without changing artifact byte-count semantics.
