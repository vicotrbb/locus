# Postulate 0223: Remote-Free Service Telemetry Summary Validation

Date: 2026-07-03

## Claim

A validation command can use `collection-summary.json` as the entrypoint for a
remote-free service telemetry evidence bundle, verify every listed artifact
byte count against the filesystem, and then run the existing manifest-backed
timing stability check without changing the underlying evidence contract.

## Rationale

Experiment 0230 added a compact JSON index to evidence bundles. That makes
bundles easier to inspect, but the index is only useful if validation tooling
can prove that the listed files still match the recorded sizes and then reuse
the manifest path that already gates timings behind counter stability.

The JSON summary should remain an index. The text manifest should remain the
source of run roles and output paths for stability validation.

## Test

Add a parser and byte-count verifier for `collection-summary.json`, plus a
command that accepts the summary path:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- <collection-summary.json>
```

The command should:

- parse and validate the summary schema;
- reject artifact paths that are absolute or escape the bundle directory;
- verify each listed artifact byte count against filesystem metadata;
- report the number of verified artifacts and total verified bytes;
- find `manifest.txt` through the summary artifacts;
- run the existing manifest-backed remote-free service telemetry stability
  check.

Validate it against the real repeated direct-capture bundle from Experiment
0230.

## Expected Outcome

The postulate survives if the summary validation command verifies all listed
artifact byte counts from the real bundle and prints the same stable timing
range as the bundle's manifest-backed validation summary.
