# Experiment 0234: Remote-Free Service Telemetry Summary Rollup Artifact

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0226-remote-free-service-telemetry-summary-rollup-artifact.md`

The postulate said that the remote-free service telemetry directory validator
could persist a compact JSON rollup artifact at the evidence root so benchmark
dashboards and release checks can consume validation status without rerunning
the directory scanner.

## Change

Extended directory mode with opt-in rollup artifact writing:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir <evidence-root> --write-rollup
```

The command still prints the existing rollup line, then writes
`collection-summary-rollup.json` at the evidence root and prints the artifact
path plus byte count.

The JSON artifact uses schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup.v1
```

and records root, summary count, valid bundle count, drifted summary count,
missing artifact count, other failure count, and timing range count.

## Commands

```text
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
sed -n '1,120p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
find target/locus-evidence/remote-free-service-summary-json -maxdepth 2 -name collection-summary\*.json -print | sort
```

## Results

The focused example tests passed, including the artifact writer test:

```text
test tests::rejects_drifted_validation_summary ... ok
test tests::reports_matching_validation_summary ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test tests::writes_directory_rollup_artifact ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real evidence root produced:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=288
```

The persisted artifact was:

```text
{
  "drifted_summaries": 0,
  "missing_artifacts": 0,
  "other_failures": 0,
  "root": "target/locus-evidence/remote-free-service-summary-json",
  "schema": "locus.remote_free_service.telemetry.collection_summary_rollup.v1",
  "summaries": 1,
  "timing_ranges": 1,
  "valid_bundles": 1
}
```

The artifact byte count was:

```text
288 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

The evidence root now contains:

```text
target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

## Interpretation

The postulate survived.

The directory validator can now leave a compact machine-readable rollup beside
the evidence bundles after validation. The artifact preserves the same clean
status as the terminal rollup: one summary, one valid bundle, zero drift, zero
missing artifacts, zero other failures, and one timing range.

## Next Question

Can the rollup artifact include a small per-bundle table with run ids and
validation status while still staying compact enough for release checks?
