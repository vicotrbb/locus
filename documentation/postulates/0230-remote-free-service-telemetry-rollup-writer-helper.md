# Postulate 0230: Remote-Free Service Telemetry Rollup Writer Helper

Date: 2026-07-03

## Claim

The remote-free service telemetry rollup artifact writer can move into
`locus-validate` so artifact creation and release checking share exported
schema constants, bundle status labels, and typed rollup data.

## Rationale

Experiment 0237 moved artifact-only release checking into the library, but the
example still owns artifact creation. That leaves schema v2 construction and
bundle status labels split between the example and library code. A public
writer helper should let CI and release tooling create the same artifact shape
that the public checker validates, while keeping the example as a thin wrapper.

## Test

Add exported rollup data types and a writer helper in
`remote_free_service_collection_summary`. The helper should write
`collection-summary-rollup.json` at the evidence root using the exported schema
and status labels. The example directory mode should delegate `--write-rollup`
to this helper.

Validate with focused library tests, the existing example tests, and a real
write plus release check against
`target/locus-evidence/remote-free-service-summary-json`.

## Expected Outcome

The postulate survives if the public writer emits an artifact accepted by the
public release-check helper, the real artifact remains 511 bytes with one valid
bundle row, and the broader workspace validation remains green.
