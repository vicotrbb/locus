# Postulate 0231: Remote-Free Service Telemetry Directory Scan Helper

Date: 2026-07-03

## Claim

The recursive directory scan for remote-free service telemetry
`collection-summary.json` files can move into `locus-validate` so release
tooling can refresh evidence rollups without depending on example-only
directory traversal.

## Rationale

Experiment 0238 moved rollup artifact writing and checking into reusable
library helpers, but the example still owns directory traversal. That means a
CI wrapper can check an existing rollup artifact through the library, but it
would need to copy the example scanner to refresh a rollup from an evidence
root. A small exported scanner should keep traversal behavior shared while
leaving higher-level bundle validation policy explicit at the caller.

## Test

Add an exported scanner that recursively finds `collection-summary.json` files,
returns sorted paths, and ignores other files such as
`collection-summary-rollup.json`. The example directory mode should use the
public scanner.

Validate with focused library tests, existing example tests, and a real scan of
`target/locus-evidence/remote-free-service-summary-json`.

## Expected Outcome

The postulate survives if the public scanner finds the real bundle summary,
the example still writes and checks the 511-byte rollup artifact, and the
broader workspace validation remains green.
