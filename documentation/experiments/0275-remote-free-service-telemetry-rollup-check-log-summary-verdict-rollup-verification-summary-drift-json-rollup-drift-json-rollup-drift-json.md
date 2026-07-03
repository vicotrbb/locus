# Experiment 0275: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0267-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json.md`

The postulate said that repeated-check rollup drift checks can emit compact
verdict JSON so dashboard archives can save matched and drifted repeated
cohort-level rollup checks as structured artifacts.

## Change

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-verifier-summary-verification-rollup-verification-log.txt> <saved-verifier-summary-verification-rollup-log.txt>
```

The mode recomputes the repeated-check rollup from saved repeated-check
verdict JSON records, compares it with the archived rollup JSON, prints the
text report, then emits compact verdict JSON with:

- `schema`;
- `status`;
- `matched`;
- `expected`;
- `actual`;
- `drift`.

The Rust implementation reuses the existing typed rollup verification report
formatter because the repeated-check drift report has the same expected rollup,
actual rollup, and first drift shape as the earlier verifier-summary rollup
check report.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json/matched-rollup-verification-json.log
perl -pe 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json/drifted-record-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json/drifted-record-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json/drifted-record-rollup-verification-json.log
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
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- a matched repeated-check rollup check emits `status=matched`,
  `matched=true`, and `drift=null`;
- a controlled stale `records=1` repeated-check rollup emits
  `status=drifted`, `matched=false`, and `drift.field=records`;
- expected and actual nested rollup summaries are preserved in the JSON;
- strict verification still rejects the stale repeated-check rollup with
  `CountDrift`.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched repeated-check rollup check emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
{"actual":{"records":2},"drift":null,"expected":{"records":2},"matched":true,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1","status":"matched"}
```

The JSON line above is shortened to the fields needed for the record. The full
artifact is saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json/matched-rollup-verification-json.log
```

The controlled stale `records=1` repeated-check rollup check emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
{"actual":{"records":1},"drift":{"actual":1,"expected":2,"field":"records"},"expected":{"records":2},"matched":false,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1","status":"drifted"}
```

The full stale artifact is saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json/drifted-record-rollup-verification-json.log
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
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Repeated-check rollup drift checks now emit dashboard-ready structured verdict
artifacts. The real matched archive emits `status=matched` with `drift=null`,
while the controlled stale archive emits `status=drifted` with
`drift.field=records`, expected `2`, and actual `1`.

This is archive-verification evidence, not allocator speed evidence.

## Next Question

Can repeated-check rollup drift verdict JSON be parsed back into typed reports
so dashboard archives can recheck repeated cohort-level check artifacts?
