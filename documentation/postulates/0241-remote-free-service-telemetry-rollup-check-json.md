# Postulate 0241: Remote-Free Service Telemetry Rollup Check JSON

Date: 2026-07-03

## Claim

Rollup release checks can emit a compact machine-readable JSON summary line
without changing the existing human-readable line.

## Rationale

The release-check line now carries enough context for human triage: schema,
artifact bytes, fingerprint, aggregate counts, host coverage, and status
coverage. A compact JSON line with the same values would make CI ingestion and
release dashboards simpler without requiring fragile token parsing.

Compatibility matters. Existing scripts may parse the first human-readable
line, so the JSON line should be additional output, not a replacement or a
change to the human line.

## Test

Add a schema-tagged compact JSON line for successful rollup release checks.

Focused tests should prove:

- the human-readable line remains unchanged;
- the JSON line is valid single-line JSON;
- the JSON fields match the release-check report;
- failed release checks still return errors instead of JSON ok output.

Real evidence should validate both current rollup artifacts and record the
compact JSON lines.

## Expected Outcome

The postulate survives if the CLI prints the existing human line first, then a
compact JSON line with the same validated report fields, while failed artifacts
retain the existing error behavior.
