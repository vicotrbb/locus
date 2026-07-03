# Experiment 0252: Remote-Free Service Telemetry Rollup Check Log Summary

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0244-remote-free-service-telemetry-rollup-check-log-summary.md`

The postulate said that saved CI logs containing multiple rollup release-check
JSON records could be parsed into a typed summary of host coverage and status
coverage.

## Change

`locus-validate` now exports
`summarize_remote_free_service_telemetry_collection_summary_rollup_check_json_log`
and `RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary`.

The summary records:

- number of parsed records;
- rollup refresh host metadata coverage;
- bundle capture host metadata coverage;
- valid, drifted, missing-artifact, and other-failure status totals.

The validation example now supports:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary <saved-log.txt>
```

That mode scans a saved log, parses every rollup-check JSON object through the
typed single-record parser, and prints one compact summary line.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary/no-host-rollup-check.log
cat target/locus-evidence/remote-free-service-rollup-check-log-summary/host-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary/no-host-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary/combined-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary target/locus-evidence/remote-free-service-rollup-check-log-summary/combined-rollup-check.log
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
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The three new log-summary tests prove:

- a two-record log reports two records;
- rollup host coverage and bundle host coverage are summed;
- status coverage is summed;
- logs without JSON records are rejected;
- schema-drifted records are rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real combined log was created from the host-bearing rollup and the older
no-host-bundle rollup. The summary command reported:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log=ok records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
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
test result: ok. 116 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Saved CI logs can now produce a compact job-level release-check evidence
summary without rereading rollup artifacts. This gives dashboards a stable
host-coverage and status-coverage line while preserving strict per-record JSON
validation.

## Next Question

Can the saved-log summary emit a JSON line with the same grouped coverage
fields for dashboard ingestion?
