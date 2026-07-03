# Experiment 0238: Remote-Free Service Telemetry Rollup Writer Helper

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0230-remote-free-service-telemetry-rollup-writer-helper.md`

The postulate said that the remote-free service telemetry rollup artifact
writer could move into `locus-validate` so artifact creation and release
checking share exported schema constants, bundle status labels, and typed
rollup data.

## Change

Added exported rollup creation types:

```text
RemoteFreeServiceTelemetryCollectionSummaryRollup
RemoteFreeServiceTelemetryCollectionSummaryRollupBundle
RemoteFreeServiceTelemetryCollectionSummaryRollupBundleStatus
```

Added the exported writer helper:

```text
write_remote_free_service_telemetry_collection_summary_rollup_artifact
```

The example directory mode now converts its directory scan result into the
exported rollup type and delegates `--write-rollup` to the library writer. The
public release-check helper from Experiment 0237 validates the artifact written
by the new public writer.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate rollup -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
cargo test -p locus-validate collection_summary -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
sed -n '1,160p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The focused library rollup tests passed:

```text
test remote_free_service_collection_summary::tests::rejects_failed_collection_summary_rollup_rows ... ok
test remote_free_service_collection_summary::tests::writes_collection_summary_rollup_artifact_for_release_check ... ok
test remote_free_service_collection_summary::tests::rejects_collection_summary_rollup_count_drift ... ok
test remote_free_service_collection_summary::tests::validates_collection_summary_rollup_artifact ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
```

The example tests still passed after delegating artifact writing to the library:

```text
test tests::rejects_drifted_validation_summary ... ok
test tests::reports_matching_validation_summary ... ok
test tests::writes_directory_rollup_artifact ... ok
test tests::rejects_failed_rollup_bundle_rows ... ok
test tests::validates_clean_rollup_artifact ... ok
test tests::rejects_rollup_count_drift ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real evidence root wrote the same compact artifact:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=511
```

The public release-check path accepted the artifact:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The persisted artifact still contained one valid bundle row and stayed at:

```text
511 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

## Fix During Validation

The first clippy run rejected repeated error-source match arms in
`RemoteFreeServiceTelemetryCollectionSummaryRollupError`. Merging the identical
arms preserved behavior and made the strict clippy command pass.

## Interpretation

The postulate survived.

Rollup artifact creation and release checking now share exported schema,
status labels, typed rollup data, and public helper APIs. The real artifact
round trip remained unchanged: one summary, one valid bundle, one timing range,
one bundle row, and 511 bytes.

## Next Question

Can the directory scanner itself be promoted into a reusable library helper so
release tooling can choose between refreshing evidence and checking persisted
rollups without depending on example-only directory traversal?
