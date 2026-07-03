# Postulate 0229: Remote-Free Service Telemetry Rollup Library Helper

Date: 2026-07-03

## Claim

The artifact-only remote-free service telemetry rollup release check can be
promoted into the `locus-validate` library so CI wrappers and release tools do
not depend on example-only code.

## Rationale

Experiment 0236 proved that the example command can validate a persisted
rollup artifact without rescanning the evidence tree. The next integration risk
is that downstream automation would need to shell out to an example binary or
copy parser logic. A public library helper should preserve the same release
check behavior while exposing typed errors and a displayable success report.

## Test

Move the schema v2 rollup artifact check into
`remote_free_service_collection_summary` and export it from `locus-validate`.
The helper should:

- read a rollup artifact path;
- parse schema v2;
- verify aggregate counts against bundle rows;
- reject failed bundle rows;
- return a displayable check report.

Validate the helper with focused library tests, keep the example `--rollup`
mode working through the public API, and run it against the real
`target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json`.

## Expected Outcome

The postulate survives if library tests reject failed rows and count drift, the
example still reports the real artifact as clean, and the broader workspace
validation stays green.
