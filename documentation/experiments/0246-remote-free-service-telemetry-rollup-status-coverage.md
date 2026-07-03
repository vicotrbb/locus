# Experiment 0246: Remote-Free Service Telemetry Rollup Status Coverage

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0238-remote-free-service-telemetry-rollup-status-coverage.md`

The postulate said that release-check output could report bundle status
coverage by status without rescanning evidence directories.

## Change

The release-check report now carries explicit status coverage counts:

- `status_valid_bundles`;
- `status_drifted_summaries`;
- `status_missing_artifacts`;
- `status_other_failures`.

The successful check struct also exposes the failed status counts as fields,
which are zero for passing artifacts. The failed-bundles error now includes
`valid_bundles` beside the failed status counts, so mixed valid and failed
artifacts still report their status distribution before returning the same
failed verdict.

No directory scan was added. The counts come from the existing artifact bundle
row pass inside
`validate_remote_free_service_telemetry_collection_summary_rollup_artifact`.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

Focused collection-summary tests passed:

```text
test remote_free_service_collection_summary::tests::rejects_failed_collection_summary_rollup_rows ... ok
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The failed-row fixture now contains one valid row and one drifted row. It still
returns `FailedBundles`, and the error string includes the mixed status
coverage:

```text
valid_bundles=1 drifted_summaries=1
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The host-bearing real rollup artifact reports clean status coverage:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=1 bundle_hosts_missing=0 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

The older no-host-bundle real rollup artifact reports the same clean status
coverage while preserving its host coverage result:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=0 bundle_hosts_missing=1 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

Final broad gates passed:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived.

Release-check output now exposes status distribution from the persisted
artifact itself. Clean artifacts get explicit zero failed counts, and failed
artifacts keep failing for the same reason while carrying their mixed status
coverage in the error output.

## Next Question

Can release-check output report artifact byte count and schema version context
without weakening artifact validation?
