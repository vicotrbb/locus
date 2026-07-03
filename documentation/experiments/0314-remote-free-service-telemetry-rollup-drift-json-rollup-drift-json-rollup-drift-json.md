# Experiment 0314: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0306-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift reports can emit
compact JSON verdicts so release dashboards can preserve typed rollup recheck
outcomes.

## Change

No Rust code change was needed. The validation example already exposes JSON
verdict emission and parser-only reload modes for saved rollup recheck
outcomes:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-verifier-summary-verification-log.txt> <saved-verifier-summary-verification-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-rollup-recheck-verdict-log.txt>
```

The formatter path uses:

```text
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_line
```

The parser path uses:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification_json_log
```

## Commands

```text
mkdir -p target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-parser/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
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

The matched JSON verdict emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The matched compact JSON preserved these fields:

```text
"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1"
"status":"matched"
"matched":true
"drift":null
"expected.records":2
"actual.records":2
```

The stale JSON verdict emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The stale compact JSON preserved these fields:

```text
"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1"
"status":"drifted"
"matched":false
"drift":{"field":"records","expected":2,"actual":1}
"expected.records":2
"actual.records":1
```

Parser-only reloads preserved the same human verdicts:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
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
rollup drift verdict rollup drift reports emit compact JSON verdicts. The
matched artifact emits and reloads `status=matched`, `matched=true`, and
`drift=null`; the stale artifact emits and reloads `status=drifted`,
`matched=false`, and `field=records` with expected `2` and actual `1`.

This is dashboard verdict cohort archive drift verdict cohort rollup drift
verdict cohort rollup drift verdict cohort rollup drift verdict artifact
evidence, not allocator speed evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict JSON records reload as typed reports
so release dashboards can recheck stored rollup recheck outcomes?
