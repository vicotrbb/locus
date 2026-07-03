# Experiment 0251: Remote-Free Service Telemetry Rollup Check JSON Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0243-remote-free-service-telemetry-rollup-check-json-parser.md`

The postulate said that release-check JSON lines could be parsed back into the
typed `RemoteFreeServiceTelemetryCollectionSummaryRollupCheck` report so saved
CI logs can be rechecked without rereading the rollup artifact.

## Change

`locus-validate` now exports
`parse_remote_free_service_telemetry_collection_summary_rollup_check_json_line`.

The parser:

- requires schema
  `locus.remote_free_service.telemetry.collection_summary_rollup_check.v1`;
- reconstructs the typed rollup-check report from flat fields;
- verifies top-level status fields match the report;
- verifies grouped `artifact`, `counts`, `host_coverage`, and
  `status_coverage` fields match the same report.

The validation example now supports:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json <saved-log.txt>
```

That mode scans a saved log for the JSON object, parses it, and prints the
reconstructed typed check line.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
mkdir -p target/locus-evidence/remote-free-service-rollup-check-json-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-json-parser/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json target/locus-evidence/remote-free-service-rollup-check-json-parser/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-json-parser/no-host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json target/locus-evidence/remote-free-service-rollup-check-json-parser/no-host-rollup-check.log
cargo fmt --all --check
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The five new parser tests prove:

- formatting then parsing a check report round-trips to the same typed report;
- schema drift is rejected;
- a missing grouped object is rejected;
- grouped numeric count drift is rejected;
- grouped string field drift is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The example test now also parses a saved two-line log containing the human
line followed by the JSON line.

During validation, clippy first rejected the parser as too large and found a
helper after the test module. The parser was split into schema, flat-field,
top-level status, and grouped-field helpers, and the helper was moved before
the test module. Final clippy passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
```

The host-bearing saved log recheck reconstructed this typed line without
rereading the rollup artifact:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=694 artifact_fingerprint=fnv1a64:82185294cde2c506 summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=1 bundle_hosts_missing=0 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

The older no-host-bundle saved log recheck reconstructed this typed line
without rereading the rollup artifact:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json schema=locus.remote_free_service.telemetry.collection_summary_rollup.v2 artifact_bytes=591 artifact_fingerprint=fnv1a64:f788b8ab364b6e1b summaries=1 valid_bundles=1 timing_ranges=1 bundles=1 rollup_host_present=true bundle_hosts=0 bundle_hosts_missing=1 status_valid_bundles=1 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

Final broad gates passed:

```text
cargo fmt --all --check
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
test result: ok. 113 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Saved release-check JSON lines can now be treated as typed evidence records.
This does not replace artifact validation because the parser does not reread
or hash the artifact. It does let CI and release systems recheck saved logs for
schema, type, and internal consistency after the artifact workspace is gone.

## Next Question

Can rollup release-check saved-log parsing support multiple JSON records and
summarize their host and status coverage across a CI job log?
