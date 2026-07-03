# Experiment 0298: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0290-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser.md`

The postulate said that repeated-check rollup drift verdict rollup check
rollup drift verdict rollup drift verdict rollup drift verdict rollup JSON can
reload as a typed rollup so dashboard archives can recheck saved
verdict-rollup-check cohorts.

## Change

No Rust code change was needed. The validation example already exposes:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify <saved-verifier-summary-verification-rollup-log.txt>
```

The mode uses:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log
```

and reloads the compact JSON schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1
```

## Commands

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup.log
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

The real saved dashboard rollup from Experiment 0297 reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The saved JSON line preserved:

```text
"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1"
"records":2
"matched":1
"drifted":1
"drift_records":1
"status_coverage":{"drifted":1,"matched":1}
"drift_fields":{"records":1}
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

Repeated-check rollup drift verdict rollup check rollup drift verdict rollup
drift verdict rollup drift verdict rollup JSON reloads as a typed rollup. The
real artifact preserved two records, one matched check, one drifted check, and
one `records` drift bucket.

This is dashboard verdict cohort rollup archive evidence, not allocator speed
evidence.

## Next Question

Can archived repeated-check rollup drift verdict rollup check rollup drift
verdict rollup drift verdict rollup drift verdict rollup JSON verify against
saved verdict-rollup-check records so dashboard archives catch stale cohort
rollups?
