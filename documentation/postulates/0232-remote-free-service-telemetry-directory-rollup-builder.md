# Postulate 0232: Remote-Free Service Telemetry Directory Rollup Builder

Date: 2026-07-03

## Claim

Full remote-free service telemetry directory rollup aggregation can move into
`locus-validate` while keeping benchmark-output stability recomputation
explicit and testable at the caller.

## Rationale

Experiment 0239 moved directory scanning into the library, but the example
still owns aggregate rollup construction. A reusable builder can scan sorted
collection summaries, invoke caller-provided bundle validation, and aggregate
valid, drifted, missing, and other failure rows into the exported rollup type.
The caller can still own expensive stability recomputation and saved-summary
comparison, which keeps benchmark-specific policy visible and testable.

## Test

Add an exported directory rollup builder that:

- scans the evidence root for sorted `collection-summary.json` paths;
- calls a caller-provided validator for each summary path;
- converts each result into a relative rollup bundle row;
- aggregates summary, valid, drifted, missing, other failure, and timing range
  counts;
- returns the exported rollup type used by the writer and release checker.

The example directory mode should delegate rollup aggregation to the public
builder while keeping manifest stability recomputation in its local validator.

## Expected Outcome

The postulate survives if focused library tests prove sorted aggregation from
an explicit validator, the real evidence root still writes a 511-byte rollup
with one valid bundle row, and the public release checker accepts it.
