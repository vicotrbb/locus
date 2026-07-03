# Postulate 0228: Remote-Free Service Telemetry Rollup Release Check

Date: 2026-07-03

## Claim

A release check can validate the remote-free service telemetry rollup artifact
directly and reject drifted, missing, or otherwise failed bundle rows without
rescanning the evidence tree.

## Rationale

Experiment 0235 made the rollup artifact detailed enough to identify each
bundle status. Release checks should be able to consume that compact artifact
as their fast path. This keeps the heavier directory scan available for
creating or refreshing evidence while making repeated release checks cheap and
deterministic.

## Test

Add an artifact-only mode:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup <collection-summary-rollup.json>
```

The command should parse schema v2, verify aggregate counts against the bundle
rows, reject any non-`valid` row, and print a compact ok line with summary,
valid bundle, and timing range counts.

Validate it against the real rollup artifact under
`target/locus-evidence/remote-free-service-summary-json`.

## Expected Outcome

The postulate survives if the real artifact-only check accepts the current
511-byte rollup artifact, reports one summary, one valid bundle, and one timing
range, and focused tests reject drifted bundle rows and count drift.
