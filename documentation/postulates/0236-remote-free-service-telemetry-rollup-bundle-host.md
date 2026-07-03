# Postulate 0236: Remote-Free Service Telemetry Rollup Bundle Host

Date: 2026-07-03

## Claim

Directory rollup bundle rows can carry each bundle's capture host metadata
without changing release-check verdict semantics.

## Rationale

Rollup-level host metadata records where a rollup was refreshed, but it does
not say where each evidence bundle was originally captured. Experiment 0242
made `collection-summary.json` carry optional capture host metadata, and
Experiment 0243 made that metadata visible in summary validation logs. The
directory rollup should preserve the same per-bundle capture context so a
single rollup artifact can show mixed evidence hosts after multiple bundles
are aggregated.

The release checker should keep its existing job: validate schema, aggregate
counts, failed bundle statuses, and timing-range totals. Capture host metadata
is triage context and must not decide release pass or fail.

## Test

Add optional host metadata to rollup bundle rows.

The directory validator should:

- copy parsed summary host metadata into valid bundle rows;
- copy parsed summary host metadata into failed rows when the summary itself
  can still be parsed;
- omit bundle host metadata when the summary lacks it or cannot be parsed.

Focused tests should prove writer output includes per-bundle host metadata and
the release checker accepts both old bundle rows without host metadata and new
bundle rows with host metadata.

## Expected Outcome

The postulate survives if real directory rollup over the host-bearing evidence
root writes bundle-level `host` metadata, old no-host evidence still rolls up,
and artifact-only release checks keep reporting the same count-focused verdict.
