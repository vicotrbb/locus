# Postulate 0233: Remote-Free Service Telemetry Rollup Host Metadata

Date: 2026-07-03

## Claim

Remote-free service telemetry rollup refresh can record benchmark host metadata
beside the rollup counters without weakening the fast artifact-only release
check.

## Rationale

Rollup artifacts currently prove that saved evidence bundles still validate,
but they do not record the host context where the rollup was refreshed. Adding
a small metadata object to the rollup artifact can help future benchmark
triage connect evidence to the operating system, CPU architecture, and hostname
available to the refresh process. The release checker should remain focused on
schema, count consistency, bundle status, and timing-range totals, so metadata
does not become a new required gate for old artifacts or for release paths that
only need artifact integrity.

## Test

Add optional host metadata to the exported rollup type and writer.

The example directory refresh should populate metadata from the current
process environment when writing `collection-summary-rollup.json`.

The release checker should still accept both:

- older schema v2 rollup artifacts without metadata;
- new schema v2 rollup artifacts with metadata.

## Expected Outcome

The postulate survives if focused library and example tests pass, the real
evidence root writes a metadata-bearing rollup artifact, and
`validate_remote_free_service_telemetry_collection_summary_rollup_artifact`
accepts it without adding metadata to the release-check report.
