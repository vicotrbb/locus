# Experiment 0239: Remote-Free Service Telemetry Directory Scan Helper

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0231-remote-free-service-telemetry-directory-scan-helper.md`

The postulate said that the recursive directory scan for remote-free service
telemetry `collection-summary.json` files could move into `locus-validate` so
release tooling can refresh evidence rollups without depending on example-only
directory traversal.

## Change

Added the exported scanner:

```text
collect_remote_free_service_telemetry_collection_summary_paths
```

The helper recursively finds `collection-summary.json` files, returns sorted
paths, and ignores other files such as `collection-summary-rollup.json`. The
example directory mode now uses this public scanner before validating bundles
and writing rollup artifacts.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
sed -n '1,160p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
find target/locus-evidence/remote-free-service-summary-json -maxdepth 2 -name 'collection-summary*.json' -print | sort
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The focused collection-summary tests passed, including the new scanner test:

```text
test remote_free_service_collection_summary::tests::scans_collection_summary_paths_sorted ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The example tests still passed after switching directory mode to the public
scanner:

```text
test tests::reports_matching_validation_summary ... ok
test tests::rejects_drifted_validation_summary ... ok
test tests::validates_clean_rollup_artifact ... ok
test tests::writes_directory_rollup_artifact ... ok
test tests::rejects_failed_rollup_bundle_rows ... ok
test tests::rejects_rollup_count_drift ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real evidence root still produced a clean rollup:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=511
```

The public release-check path accepted the scanner-backed artifact:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The evidence root contained:

```text
target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

The artifact stayed at:

```text
511 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

The broader gate passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Interpretation

The postulate survived.

Directory traversal is now reusable through `locus-validate`. Release tooling
can scan an evidence root, feed the results into refresh logic, write a public
rollup artifact, and check a persisted rollup without copying example-only
scanner code.

## Next Question

Can full directory rollup validation move into a reusable library helper while
keeping the benchmark-output stability recomputation explicit and testable?
