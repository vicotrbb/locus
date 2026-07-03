# Experiment 0286: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0278-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser.md`

The postulate said that repeated-check rollup drift verdict rollup check rollup
drift verdict JSON can be parsed back into typed reports so dashboard archives
can recheck saved rollup-check verdict artifacts.

## Change

The validation example exposes:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-verifier-summary-verification-rollup-verification-log.txt>
```

The mode reuses the shared typed parser for:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1
```

Existing focused tests parse matched and drifted repeated verdict rollup check
rollup drift verdict JSON back into typed reports.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-rollup-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser/matched-rollup-verification-json-parse.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/drifted-record-rollup-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser/drifted-record-rollup-verification-json-parse.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The parser tests prove:

- matched repeated-check rollup drift verdict rollup check rollup drift verdict
  JSON reloads as the original typed report;
- drifted JSON reloads with `field=records`;
- expected and actual nested rollups survive the parse.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched verdict JSON reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The controlled stale `records=1` verdict JSON reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The parsed artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser/matched-rollup-verification-json-parse.log
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser/drifted-record-rollup-verification-json-parse.log
```

Final broad gates passed:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

The full workspace suite reported:

```text
test result: ok. 191 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 198 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Repeated-check rollup drift verdict rollup check rollup drift verdict JSON can
be reloaded as typed reports. The real matched artifact remains matched, while
the controlled stale `records=1` artifact remains drifted with
`drift.field=records`, expected `2`, and actual `1`.

This is archive-recheck evidence, not allocator speed evidence.

## Next Question

Can repeated-check rollup drift verdict rollup check rollup drift verdict JSON
records be aggregated into a dashboard rollup so saved rollup-check verdict
outcomes can be summarized?
