# Experiment 0265: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0257-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json.md`

The postulate said that verifier-summary drift checks can emit compact verdict
JSON so dashboard archives can save matched and drifted aggregate-summary
checks as structured artifacts.

## Change

`locus-validate` now exports:

```text
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_json_line
```

The formatter emits schema
`locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification.v1`
with:

- `status`;
- `matched`;
- `expected`;
- `actual`;
- `drift`.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json <saved-verifier-log.txt> <saved-verifier-summary-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/matched-summary-verification-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift/drifted-record-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/drifted-record-summary-verification-json.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- a matched verifier-summary drift check emits `status=matched`,
  `matched=true`, and `drift=null`;
- a controlled stale `records=1` verifier summary emits `status=drifted`,
  `matched=false`, and `drift.field=records`;
- expected and actual nested summary counters are preserved in the JSON;
- the strict verifier still rejects the stale summary with `CountDrift`.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched verifier-summary archive emitted a structured verdict:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
{"actual":{"drift_fields":{"records":1},"records":2},"drift":null,"expected":{"drift_fields":{"records":1},"records":2},"matched":true,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification.v1","status":"matched"}
```

The JSON line above is shortened to the fields needed for the record. The full
artifact is saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/matched-summary-verification-json.log
```

The controlled stale `records=1` summary emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
{"actual":{"records":1},"drift":{"actual":1,"expected":2,"field":"records"},"expected":{"records":2},"matched":false,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification.v1","status":"drifted"}
```

The full stale artifact is saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json/drifted-record-summary-verification-json.log
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
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Verifier-summary drift checks now produce dashboard-ready structured verdict
artifacts. The real archive emits `status=matched` with `drift=null`, while the
controlled stale archive emits `status=drifted` with `drift.field=records`,
expected `2`, and actual `1`.

This is an archive-verification result, not allocator speed evidence. Its value
is that future benchmark dashboards can preserve aggregate-summary validation
as structured data.

## Next Question

Can verifier-summary drift verdict JSON be parsed back into typed reports so
dashboard archives can recheck aggregate-summary verdict artifacts?
