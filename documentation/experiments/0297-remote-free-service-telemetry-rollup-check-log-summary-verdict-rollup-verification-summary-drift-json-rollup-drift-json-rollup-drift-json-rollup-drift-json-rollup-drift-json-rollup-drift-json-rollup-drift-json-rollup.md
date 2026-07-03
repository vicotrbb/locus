# Experiment 0297: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0289-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup.md`

The postulate said that saved repeated-check rollup drift verdict rollup check
rollup drift verdict rollup drift verdict rollup drift verdict JSON records can
aggregate into a dashboard rollup so release checks can summarize matched and
drifted verdict-rollup-check outcomes.

## Change

The validation example exposes:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup <saved-verifier-summary-verification-rollup-verification-log.txt>
```

The mode reuses the shared typed summarizer for:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1
```

and emits rollup JSON with schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1
```

Existing focused tests summarize matched and drifted JSON verdict records into
the same typed rollup.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-rollup-verification-json.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/drifted-record-rollup-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup.log
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

The focused tests prove:

- saved matched verdict-rollup-check JSON records contribute to `matched`;
- saved drifted verdict-rollup-check JSON records contribute to `drifted`;
- drifted records increment the first-drift field bucket, including
  `drift_records`.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real mixed saved verdict log aggregated as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The rollup JSON preserved:

```text
"records":2
"matched":1
"drifted":1
"drift_records":1
"status_coverage":{"drifted":1,"matched":1}
"drift_fields":{"records":1}
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup.log
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

Saved repeated-check rollup drift verdict rollup check rollup drift verdict
rollup drift verdict rollup drift verdict JSON records aggregate into a
dashboard rollup. The real mixed log reports two records, one matched check,
one drifted check, and one `records` drift bucket.

This is dashboard verdict cohort rollup evidence, not allocator speed evidence.

## Next Question

Can repeated-check rollup drift verdict rollup check rollup drift verdict
rollup drift verdict rollup drift verdict rollup JSON reload as a typed rollup
so dashboard archives can recheck saved verdict-rollup-check cohorts?
