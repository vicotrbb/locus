# Experiment 0236: Remote-Free Service Telemetry Rollup Release Check

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0228-remote-free-service-telemetry-rollup-release-check.md`

The postulate said that a release check could validate the remote-free service
telemetry rollup artifact directly and reject drifted, missing, or otherwise
failed bundle rows without rescanning the evidence tree.

## Change

Added artifact-only rollup validation:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup <collection-summary-rollup.json>
```

The check parses schema v2, verifies aggregate counts against the `bundles`
rows, rejects unknown statuses, rejects drifted, missing, or otherwise failed
rows, and prints a compact ok line for clean artifacts.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
sed -n '1,160p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo test -p locus-validate collection_summary -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The focused example tests passed:

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

The real directory rollup regenerated the artifact:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=511
```

The artifact-only release check accepted the real artifact:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The persisted artifact stayed at:

```text
511 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
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

The validator now has a cheap release-check path for the rollup artifact. It
does not rescan the evidence tree and still rejects failed bundle rows and
aggregate count drift. The real current artifact reports one summary, one valid
bundle, one timing range, and remains 511 bytes.

## Next Question

Can the artifact-only release check be promoted into a reusable library helper
so CI wrappers do not depend on example-only code?
