# Experiment 0240: Remote-Free Service Telemetry Directory Rollup Builder

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0232-remote-free-service-telemetry-directory-rollup-builder.md`

The postulate said that full remote-free service telemetry directory rollup
aggregation could move into `locus-validate` while the caller keeps explicit
ownership of benchmark-output stability recomputation and saved-summary
comparison.

## Change

Added the exported builder:

```text
build_remote_free_service_telemetry_collection_summary_directory_rollup
```

The helper scans sorted `collection-summary.json` paths, calls a caller-owned
validator for each summary, converts each validator result into a relative
bundle row, and aggregates the public rollup counters. The caller provides
`RemoteFreeServiceTelemetryCollectionSummaryBundleValidation`, so the example
still owns manifest parsing, Criterion output stability recomputation, and
validation-summary drift classification.

The rollup type also gained a `Display` implementation so library-built
rollups print the same stable one-line status as the example used before.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
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

The focused library tests passed, including the new directory rollup builder
test over real temporary files:

```text
test remote_free_service_collection_summary::tests::builds_collection_summary_directory_rollup_from_validator ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The example tests still passed with directory aggregation delegated to the
public builder:

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

The real evidence root still produced the expected rollup:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=511
```

The public release-check path accepted the builder-backed artifact:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The artifact content stayed compact and schema-compatible:

```text
{
  "bundles": [
    {
      "run_id": "apply-confirm-summary-1783084007-13676",
      "status": "valid",
      "summary": "apply-confirm-summary-1783084007-13676/collection-summary.json",
      "timing_ranges": 1
    }
  ],
  "drifted_summaries": 0,
  "missing_artifacts": 0,
  "other_failures": 0,
  "root": "target/locus-evidence/remote-free-service-summary-json",
  "schema": "locus.remote_free_service.telemetry.collection_summary_rollup.v2",
  "summaries": 1,
  "timing_ranges": 1,
  "valid_bundles": 1
}
```

The artifact byte count stayed unchanged:

```text
511 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

The first clippy run caught a needless borrow in the example wrapper. The code
was fixed, and the final broad gate passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived.

Directory rollup aggregation is now reusable through `locus-validate`, while
benchmark-specific recomputation remains explicit in the caller. This keeps the
library boundary small: it owns deterministic filesystem scanning, relative
bundle row construction, counter aggregation, overflow checking, artifact
writing, and artifact checking. The example owns Criterion output parsing and
semantic validation of saved benchmark summaries.

## Next Question

Can the directory rollup refresh path record benchmark host metadata beside the
rollup without weakening the fast artifact-only release check?
