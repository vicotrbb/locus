# Experiment 0266: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0258-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-parser.md`

The postulate said that verifier-summary drift verdict JSON can be parsed back
into typed reports so dashboard archives can recheck aggregate-summary verdict
artifacts.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_log
```

The line parser reloads the embedded expected and actual verifier summaries,
checks their grouped counters, recomputes the first drift, and rejects verdict
JSON whose `status`, `matched`, or `drift` payload disagrees with the embedded
summaries.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-verify <saved-verifier-summary-verification-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/matched-summary-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-parser/matched-summary-verification-json-parse.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/drifted-record-summary-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-parser/drifted-record-summary-verification-json-parse.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 76 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- matched verifier-summary verdict JSON parses back into the original typed
  report;
- drifted verifier-summary verdict JSON parses back with `field=records`;
- tampered `status` fields are rejected;
- tampered `drift` payloads are rejected;
- nested expected summary group drift is rejected before the verdict report is
  accepted.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched verifier-summary verdict JSON artifact reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The controlled stale `records=1` verifier-summary verdict JSON artifact
reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

Parsed evidence was saved under:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-parser/
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
test result: ok. 166 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Dashboard archives can now save verifier-summary drift verdict JSON and later
reload it as a typed report. The parser recomputes drift from the embedded
summaries, so a tampered verdict cannot claim a different status or first
drift without being rejected.

This is archive-verification evidence, not allocator speed evidence.

## Next Question

Can verifier-summary drift verdict JSON records be aggregated into a dashboard
rollup so repeated aggregate-summary checks have cohort-level status and drift
coverage?
