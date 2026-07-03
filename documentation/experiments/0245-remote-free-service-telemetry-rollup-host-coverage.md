# Experiment 0245: Remote-Free Service Telemetry Rollup Host Coverage

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0237-remote-free-service-telemetry-rollup-host-coverage.md`

The postulate said that release checks could report rollup host and bundle
host coverage counts without making host metadata part of the release-check
verdict.

## Change

The release-check report now includes three non-verdict fields:

- `rollup_host_present`;
- `bundle_hosts`;
- `bundle_hosts_missing`.

The checker treats host metadata as coverage only when the `host` value is a
JSON object. Missing host metadata and non-object host metadata are not
errors. Verdict semantics still come only from schema, aggregate counts,
failed bundle statuses, timing ranges, and bundle row count.

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

Focused collection-summary tests passed with one new invalid-host coverage
case:

```text
test remote_free_service_collection_summary::tests::accepts_invalid_collection_summary_rollup_host_metadata_as_no_coverage ... ok
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The host-bearing rollup artifact reported full bundle host coverage:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=1 bundle_hosts_missing=0
```

The older no-host bundle artifact still passed and reported zero bundle host
coverage:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=0 bundle_hosts_missing=1
```

The failed-row fixture now carries host metadata and still fails because of the
bundle status, proving host metadata does not override verdict semantics. The
invalid-host fixture passes as valid evidence status and reports no host
coverage.

Final broad gates passed:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived.

Release-check output now exposes host coverage for dashboards and manual
triage without requiring host metadata and without changing artifact pass or
fail criteria.

## Next Question

Can release-check output report status coverage by bundle status without
rescanning evidence directories?
