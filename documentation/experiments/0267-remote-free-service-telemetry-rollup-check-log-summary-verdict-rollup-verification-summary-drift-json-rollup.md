# Experiment 0267: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0259-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup.md`

The postulate said that verifier-summary drift verdict JSON records can be
aggregated into a dashboard rollup so repeated aggregate-summary checks have
cohort-level status and drift coverage.

## Change

`locus-validate` now exports:

```text
summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line
```

The rollup uses schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1
```

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup <saved-verifier-summary-verification-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/matched-summary-verification-json.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/drifted-record-summary-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup/combined-summary-verification-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup/combined-summary-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup/combined-summary-verification-json-rollup.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 78 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- a mixed matched-plus-drifted verifier-summary verdict log rolls up to two
  records, one matched record, one drifted record, and one `records` drift
  bucket;
- the rollup JSON uses its own schema;
- grouped `status_coverage` and `drift_fields` counters match the flat fields;
- an empty log is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real mixed verifier-summary drift verdict log rolled up as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
{"drift_fields":{"records":1},"drifted":1,"matched":1,"records":2,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1","status_coverage":{"drifted":1,"matched":1}}
```

The JSON line above is shortened to the fields needed for the record. The full
artifact is saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup/combined-summary-verification-json-rollup.log
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
test result: ok. 168 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Repeated verifier-summary drift verdict checks now have a dashboard rollup with
cohort-level status coverage and first-drift coverage. The real mixed archive
reports two records, one matched artifact, one drifted artifact, and one
`records` drift bucket.

This is archive-verification evidence, not allocator speed evidence.

## Next Question

Can verifier-summary drift verdict rollup JSON be parsed back into typed
reports so dashboard archives can recheck cohort-level aggregate-summary
verdict artifacts?
