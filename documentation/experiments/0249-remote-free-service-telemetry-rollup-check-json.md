# Experiment 0249: Remote-Free Service Telemetry Rollup Check JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0241-remote-free-service-telemetry-rollup-check-json.md`

The postulate said that release checks could expose a compact
machine-readable JSON summary line without changing the existing
human-readable release-check line.

## Change

Successful rollup release checks now print the existing human-readable line
first, then a compact single-line JSON object.

The JSON line uses schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check.v1
```

It mirrors the validated report fields that the human line already exposes:
path, accepted rollup schema, artifact bytes, artifact fingerprint, aggregate
counts, host coverage, and status coverage.

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
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The host-bearing real rollup artifact preserved the existing human line and
then printed the new compact JSON line:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=694 artifact_fingerprint=fnv1a64:82185294cde2c506 summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=1 bundle_hosts_missing=0 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
{"artifact_bytes":694,"artifact_fingerprint":"fnv1a64:82185294cde2c506","bundle_hosts":1,"bundle_hosts_missing":0,"bundles":1,"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"path":"target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json","rollup_host_present":true,"rollup_schema":"locus.remote_free_service.telemetry.collection_summary_rollup.v2","schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check.v1","status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":1,"summaries":1,"timing_ranges":1,"valid_bundles":1}
```

The older no-host-bundle real rollup artifact also preserved the existing human
line and printed the compact JSON line:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=591 artifact_fingerprint=fnv1a64:f788b8ab364b6e1b summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=0 bundle_hosts_missing=1 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
{"artifact_bytes":591,"artifact_fingerprint":"fnv1a64:f788b8ab364b6e1b","bundle_hosts":0,"bundle_hosts_missing":1,"bundles":1,"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"path":"target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json","rollup_host_present":true,"rollup_schema":"locus.remote_free_service.telemetry.collection_summary_rollup.v2","schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check.v1","status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":1,"summaries":1,"timing_ranges":1,"valid_bundles":1}
```

Final broad gates passed:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

The full workspace suite reported:

```text
test result: ok. 191 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Release-check output now has both stable surfaces:

- the existing first line for human logs and existing token parsers;
- a schema-tagged JSON line for CI ingestion, release dashboards, and future
  evidence rollups.

The JSON output is derived from the validated report after artifact parsing and
status checks, so failed rollups still return errors rather than an ok JSON
record.

## Next Question

Can release-check JSON lines include stable nested groups for artifact, host
coverage, and status coverage without breaking the compact flat line?
