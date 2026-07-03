# Postulate 0226: Remote-Free Service Telemetry Summary Rollup Artifact

Date: 2026-07-03

## Claim

The remote-free service telemetry directory validator can persist a compact
JSON rollup artifact at the evidence root so benchmark dashboards and release
checks can consume validation status without rerunning the directory scanner.

## Rationale

Experiment 0233 added directory rollup validation, but the result only exists
as command output. That is enough for a terminal workflow and weak for later
automation. A small JSON artifact with the same counts should make the evidence
directory self-describing after validation while keeping the existing text
output stable.

## Test

Extend directory mode with an opt-in artifact write:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir <evidence-root> --write-rollup
```

The command should:

- validate the directory exactly like the existing `--dir` mode;
- write `collection-summary-rollup.json` at the evidence root;
- include a schema string, root, summary count, valid bundle count, drifted
  summary count, missing artifact count, other failure count, and timing range
  count;
- print the artifact path and byte count after writing it.

Validate it against the real `remote-free-service-summary-json` evidence root.

## Expected Outcome

The postulate survives if the real directory rollup writes a JSON artifact with
one summary, one valid bundle, zero drifted summaries, zero missing artifacts,
zero other failures, and one timing range.
