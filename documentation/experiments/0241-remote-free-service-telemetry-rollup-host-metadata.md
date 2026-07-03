# Experiment 0241: Remote-Free Service Telemetry Rollup Host Metadata

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0233-remote-free-service-telemetry-rollup-host-metadata.md`

The postulate said that remote-free service telemetry rollup refresh could
record benchmark host metadata beside the rollup counters without weakening
the fast artifact-only release check.

## Change

Added optional host metadata to the exported rollup type:

```text
RemoteFreeServiceTelemetryCollectionSummaryRollupHost
```

The metadata records:

- Rust target operating system;
- Rust target CPU architecture;
- hostname when the refresh process exposes `HOSTNAME` or `COMPUTERNAME`.

The writer includes a `host` object only when the rollup carries metadata. The
directory validation example populates host metadata before writing
`collection-summary-rollup.json`.

The release checker still validates schema, count consistency, bundle status,
and timing-range totals. It accepts old artifacts without `host` and new
artifacts with `host`, and it does not include metadata in the release-check
report.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
sed -n '1,180p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The focused library tests passed. They now include both a legacy artifact
without metadata and a metadata-bearing artifact through the same public
release checker:

```text
test remote_free_service_collection_summary::tests::validates_collection_summary_rollup_artifact ... ok
test remote_free_service_collection_summary::tests::validates_collection_summary_rollup_artifact_with_host_metadata ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The example tests passed after directory refresh started attaching host
metadata:

```text
test tests::reports_matching_validation_summary ... ok
test tests::rejects_drifted_validation_summary ... ok
test tests::writes_directory_rollup_artifact ... ok
test tests::validates_clean_rollup_artifact ... ok
test tests::rejects_failed_rollup_bundle_rows ... ok
test tests::rejects_rollup_count_drift ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real evidence root wrote a metadata-bearing rollup:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=591
```

The public release-check path accepted the new artifact while keeping its
output focused on counts:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The real artifact now records host metadata:

```text
"host": {
  "arch": "aarch64",
  "hostname": null,
  "os": "macos"
}
```

The artifact byte count is now:

```text
591 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

The first broad clippy run caught a documented-panic issue from an `expect`
used while inserting the host object. The writer was changed to insert through
`if let Some(object) = artifact.as_object_mut()`, and the final clippy run
passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
```

The benchmark compile and full workspace test gates passed:

```text
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived.

The rollup refresh path can now carry host context for benchmark triage without
making host metadata part of the release-check verdict. Old schema v2 rollups
without `host` still validate, and new schema v2 rollups with `host` validate
through the same artifact-only check.

## Next Question

Can the evidence bundle summary itself record the direct capture host metadata
so per-bundle context survives even when rollups are regenerated elsewhere?
