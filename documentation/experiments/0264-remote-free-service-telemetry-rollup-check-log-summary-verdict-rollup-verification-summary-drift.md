# Experiment 0264: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0256-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift.md`

The postulate said that archived verifier-summary JSON could be checked
against the saved verifier JSON records it claims to summarize, so dashboard
archives can detect stale aggregate verifier summaries.

## Change

`locus-validate` now exports:

```text
check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log
verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_log
```

The check helper returns a typed drift report. The strict verifier returns the
archived summary on match and rejects drift with `CountDrift`. The validation
example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against <saved-verifier-log.txt> <saved-verifier-summary-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-report <saved-verifier-log.txt> <saved-verifier-summary-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift/matched-summary-verification.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-report target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift/matched-summary-verification-report.log
perl -pe 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift/drifted-record-summary.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-report target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift/drifted-record-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift/drifted-record-summary-verification-report.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- a verifier summary recomputed from saved verifier JSON records matches the
  archived verifier-summary JSON;
- a controlled stale `records=1` archived summary reports `records` drift;
- the strict verifier rejects the stale summary with `CountDrift`;
- grouped summary drift is rejected by the summary parser before comparison.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched verifier-summary archive checked against its saved verifier
records returned:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The diagnostic report for the same archive returned:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The controlled stale `records=1` summary reported:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The strict verifier rejected the stale archive with:

```text
CountDrift { field: "records", expected: 2, actual: 1 }
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
test result: ok. 159 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Dashboard archives can now detect stale aggregate verifier summaries by
recomputing them from saved verifier JSON records. A controlled stale
`records=1` summary is rejected with expected `records=2` and actual
`records=1`.

## Next Question

Can verifier-summary drift checks emit compact verdict JSON so dashboard
archives can save matched and drifted aggregate-summary checks as structured
artifacts?
