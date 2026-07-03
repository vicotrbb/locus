# Experiment 0237: Remote-Free Service Telemetry Rollup Library Helper

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0229-remote-free-service-telemetry-rollup-library-helper.md`

The postulate said that the artifact-only remote-free service telemetry rollup
release check could be promoted into the `locus-validate` library so CI
wrappers and release tools do not depend on example-only code.

## Change

Moved the schema v2 rollup artifact check into
`remote_free_service_collection_summary` and exported it from `locus-validate`
as:

```text
validate_remote_free_service_telemetry_collection_summary_rollup_artifact
```

The public API returns
`RemoteFreeServiceTelemetryCollectionSummaryRollupCheck` on success and
`RemoteFreeServiceTelemetryCollectionSummaryRollupError` on failure. The helper
reads the artifact path, parses schema v2, verifies aggregate counts against
bundle rows, rejects unknown statuses, rejects failed bundle rows, and exposes
the same display line used by the example command.

The example `--rollup` mode now delegates to the public helper.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate rollup -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo test -p locus-validate collection_summary -- --nocapture
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
test remote_free_service_collection_summary::tests::rejects_collection_summary_rollup_count_drift ... ok
test remote_free_service_collection_summary::tests::validates_collection_summary_rollup_artifact ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
```

The example tests still passed after delegating to the library helper:

```text
test tests::reports_matching_validation_summary ... ok
test tests::rejects_drifted_validation_summary ... ok
test tests::writes_directory_rollup_artifact ... ok
test tests::validates_clean_rollup_artifact ... ok
test tests::rejects_rollup_count_drift ... ok
test tests::rejects_failed_rollup_bundle_rows ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real artifact-only command still accepted the persisted rollup:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The broader gate passed:

```text
cargo test -p locus-validate collection_summary -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Interpretation

The postulate survived.

CI wrappers can now consume the rollup artifact through a typed library helper
instead of depending on example-only parser code. The example command remains a
thin compatibility layer over the same public API and still accepts the real
511-byte artifact with one summary, one valid bundle, one timing range, and one
bundle row.

## Next Question

Can the rollup artifact writer also move into a reusable library helper so both
creation and release checking share exported validation types?
