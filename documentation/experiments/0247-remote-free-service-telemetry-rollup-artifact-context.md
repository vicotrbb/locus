# Experiment 0247: Remote-Free Service Telemetry Rollup Artifact Context

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0239-remote-free-service-telemetry-rollup-artifact-context.md`

The postulate said that release-check output could report artifact byte count
and schema version context without weakening artifact validation.

## Change

The release-check report now includes:

- `schema`;
- `artifact_bytes`.

The schema is the accepted schema string after normal schema validation. The
byte count is the exact length of the artifact text read by the checker before
JSON parsing.

Unsupported schemas still fail before producing an ok report.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

Focused collection-summary tests passed with the new unexpected-schema
guardrail:

```text
test remote_free_service_collection_summary::tests::rejects_unexpected_collection_summary_rollup_schema ... ok
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The host-bearing real rollup artifact reported schema and byte-count context:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=694 summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=1 bundle_hosts_missing=0 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

The older no-host-bundle real rollup artifact reported the same schema and its
own byte count:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=591 summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=0 bundle_hosts_missing=1 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

The byte counts matched `wc -c`:

```text
694 target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json
591 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
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

Release-check output now includes schema and exact artifact byte count context
for passing artifacts, while unsupported schemas remain hard validation
failures.

## Next Question

Can the release-check output expose a stable evidence fingerprint for rollup
artifacts without introducing cryptographic dependencies?
