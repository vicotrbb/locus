# Experiment 0328: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0320-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck outcomes can emit compact JSON verdicts so release
dashboards can archive aggregate rollup recheck cohort verification rollup
recheck decisions.

## Change

No Rust code change was needed. The validation example already exposes a
compact JSON verdict mode for saved aggregate rollup recheck cohort rollup
archive decisions:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

This experiment emitted compact JSON verdicts for the real saved archive from
Experiment 0327 and for a controlled stale archive with `records=1`.

## Commands

```text
mkdir -p target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
perl -pe 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-parse.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-parse.log
wc -l target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/*.log
rg -o 'status=matched|status=drifted|field=records|expected=2|actual=1|"schema":"[^"]+"|"status":"matched"|"status":"drifted"|"matched":true|"matched":false|"field":"records"|"expected":2|"actual":1|"records":2|"records":1' target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
```

## Results

The matched saved archive emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

Its compact JSON verdict used schema
`locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1`
and preserved:

```text
"status":"matched"
"matched":true
"drift":null
"actual":{"records":2}
"actual":{"drift_fields":{"records":1}}
```

The controlled stale archive emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

Its compact JSON verdict preserved:

```text
"status":"drifted"
"matched":false
"drift":{"field":"records","expected":2,"actual":1}
"actual":{"records":1}
```

Both saved verdict logs reloaded through the parser path:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The evidence directory contained 14 total log lines:

```text
4 saved source verdict log
1 matched parse log
2 matched JSON verdict log
2 matched rollup log
1 stale parse log
2 stale JSON verdict log
2 stale rollup log
14 total
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-parse.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-parse.log
```

Focused and broad validation gates passed:

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
```

Focused collection-summary tests passed:

```text
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
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
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck outcomes emit compact JSON verdicts. The real matched
archive emitted and reloaded a matched decision, while the controlled stale
archive emitted and reloaded a drifted decision with the expected `records`
drift.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict cohort rollup verdict evidence, not allocator speed evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck JSON verdict records reload as typed reports so release
dashboards can reprocess stored aggregate rollup recheck cohort verification
rollup recheck decisions?
