# Experiment 0309: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0301-remote-free-service-telemetry-rollup-drift-json-rollup-drift.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup JSON can verify against saved source verdict
records so release dashboards can reject stale cohort rollup artifacts.

## Change

No Rust code change was needed. The validation example already exposes:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-verifier-summary-verification-rollup-verification-log.txt> <saved-verifier-summary-verification-rollup-log.txt>
```

The strict mode uses:

```text
verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log
```

and the report mode uses:

```text
check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log
```

## Commands

```text
mkdir -p target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift
awk '/^\{/ { sub("\"records\":2", "\"records\":1") } { print }' target/locus-evidence/remote-free-service-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against target/locus-evidence/remote-free-service-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/matched-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verify-against.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-report target/locus-evidence/remote-free-service-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-report.log
set +e
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against target/locus-evidence/remote-free-service-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-strict.log 2>&1
STATUS=$?
set -e
printf 'stale_strict_exit=%s\n' "$STATUS" > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-strict-status.txt
test "$STATUS" -eq 1
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The real archived rollup verified as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The controlled stale `records=1` archive reported:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

Strict verification of the stale archive exited with status `1` and reported:

```text
Error: CountDrift { field: "records", expected: 2, actual: 1 }
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/matched-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verify-against.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-report.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-strict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift/stale-record-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-strict-status.txt
```

Focused collection-summary tests passed:

```text
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
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

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup JSON verifies against saved source verdict records. The real archived
rollup matches, while a controlled stale `records=1` archive fails strict
verification with `CountDrift`.

This is dashboard verdict cohort archive drift verdict cohort rollup drift
verdict cohort rollup drift check evidence, not allocator speed evidence.

## Next Question

Can repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift reports emit compact JSON verdicts so release dashboards can store
matched and stale cohort rollup check outcomes as machine readable records?
