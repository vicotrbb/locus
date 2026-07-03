# Experiment 0261: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Drift JSON Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0253-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-drift-json-parser.md`

The postulate said that verdict rollup verification JSON could be parsed back
into typed reports so dashboard archives can recheck their own drift-verdict
artifacts.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log
```

The parser validates schema, status, matched flag, expected rollup, actual
rollup, drift payload, and nested grouped rollup counters. The validation
example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-verify <saved-verdict-rollup-verification-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/combined-verdict-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/combined-verdict-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/matched-verdict-rollup-verification.log
perl -pe 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/combined-verdict-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/drifted-record-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/drifted-record-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/drifted-record-verdict-rollup-verification.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/matched-verdict-rollup-verification.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/drifted-record-verdict-rollup-verification.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- matched verification JSON round-trips back into a typed report;
- drifted verification JSON round-trips back into a typed report;
- status drift is rejected;
- drift payload mismatch is rejected;
- nested expected or actual rollup coverage drift is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched verification log parsed back into:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_rollup_hosts_present=0 drift_rollup_hosts_missing=0 drift_bundle_hosts=0 drift_bundle_hosts_missing=0 drift_status_valid_bundles=0 drift_status_drifted_summaries=0 drift_status_missing_artifacts=0 drift_status_other_failures=0
```

The real controlled stale `records=1` verification log parsed back into:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
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
test result: ok. 148 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Structured verdict rollup verification artifacts now have a self-check path.
Dashboard archives can reload matched and drifted verdict artifacts as typed
reports and reject inconsistent schema, status, drift, or nested rollup fields.

## Next Question

Can verdict rollup verification JSON records be aggregated into a compact
dashboard summary of matched and drifted rollup-verification artifacts?
