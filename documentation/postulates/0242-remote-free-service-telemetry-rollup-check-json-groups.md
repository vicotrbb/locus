# Postulate 0242: Remote-Free Service Telemetry Rollup Check JSON Groups

Date: 2026-07-03

## Claim

Rollup release-check JSON lines can add stable nested groups for artifact,
host coverage, and status coverage without removing or changing the compact
flat fields.

## Rationale

The current JSON line is easy for simple CI jobs to consume because every
validated value is a top-level field. Larger dashboards and typed ingestion
pipelines benefit from grouped fields that separate artifact identity, host
coverage, aggregate counts, and status coverage.

The flat fields should remain because they are already a compact compatibility
surface. Nested groups should be additive and should mirror the same validated
report values, not introduce alternate counters.

## Test

Add nested JSON objects to successful release-check JSON lines:

- `artifact` for path, schema, byte count, and fingerprint;
- `counts` for summary, timing-range, and bundle totals;
- `host_coverage` for rollup and bundle host coverage;
- `status_coverage` for valid and failed bundle status counts.

Focused tests should prove:

- existing top-level fields remain unchanged;
- nested fields match the same release-check report values;
- the JSON line remains a single line;
- failed release checks still return errors instead of JSON ok output.

Real evidence should validate both current rollup artifacts and record the
new grouped JSON lines.

## Expected Outcome

The postulate survives if the CLI keeps the existing human line first, keeps
the flat JSON fields intact, and adds nested groups with the same validated
values for both host-bearing and older no-host-bundle rollups.
