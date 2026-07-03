# Experiment 0248: Remote-Free Service Telemetry Rollup Fingerprint

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0240-remote-free-service-telemetry-rollup-fingerprint.md`

The postulate said that release-check output could expose a stable evidence
fingerprint for rollup artifacts without introducing cryptographic
dependencies.

## Change

The release-check report now includes `artifact_fingerprint`.

The fingerprint is a dependency-free FNV-1a 64-bit hash over the exact artifact
text read by the checker, formatted as:

```text
fnv1a64:<16 lowercase hex digits>
```

This is an evidence identity and drift-triage token. It is not a cryptographic
integrity claim.

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

Focused collection-summary tests passed with the new fingerprint test:

```text
test remote_free_service_collection_summary::tests::rollup_artifact_fingerprint_is_stable_and_content_sensitive ... ok
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The fingerprint test proves:

- repeated computation on the same artifact text is stable;
- adding host metadata changes the fingerprint;
- the string starts with `fnv1a64:`;
- the string length is 24 bytes.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The host-bearing real rollup artifact reported:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=694 artifact_fingerprint=fnv1a64:82185294cde2c506 summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=1 bundle_hosts_missing=0 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

The older no-host-bundle real rollup artifact reported:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=591 artifact_fingerprint=fnv1a64:f788b8ab364b6e1b summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=0 bundle_hosts_missing=1 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
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

Release-check output now includes a stable dependency-free artifact
fingerprint for passing rollup artifacts. The fingerprint is useful for
evidence comparison and log triage, while validation still relies on schema,
counts, status rows, timing ranges, and bundle rows.

## Next Question

Can rollup release checks expose a machine-readable compact JSON summary line
without changing the human-readable line?
