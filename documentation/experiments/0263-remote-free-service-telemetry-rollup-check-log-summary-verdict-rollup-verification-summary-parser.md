# Experiment 0263: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0255-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-parser.md`

The postulate said that saved verdict rollup verification summary JSON could
be parsed back into typed summary reports so dashboard archives can validate
aggregate verifier-summary artifacts.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log
```

The parser validates schema, flat counters, `status_coverage`, and
`drift_fields`. The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify <saved-verdict-rollup-verification-summary-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-parser/parsed-summary.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- formatted verifier-summary JSON round-trips back into a typed summary;
- saved logs can contain human-readable lines around the summary JSON;
- schema drift is rejected;
- missing grouped fields are rejected;
- grouped status coverage drift is rejected;
- grouped drift-field coverage drift is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real verifier-summary log from Experiment 0262 parsed back into:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
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
test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Dashboard archives can now reload aggregate verifier-summary artifacts as typed
reports and reject schema drift, missing grouped fields, grouped status drift,
and grouped drift-field drift.

## Next Question

Can archived verifier-summary JSON be checked against the saved verifier JSON
records it claims to summarize, so dashboard archives can detect stale
aggregate verifier summaries?
