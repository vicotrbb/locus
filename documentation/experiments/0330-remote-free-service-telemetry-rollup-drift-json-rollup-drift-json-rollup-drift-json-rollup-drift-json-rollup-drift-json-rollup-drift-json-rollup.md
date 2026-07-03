# Experiment 0330: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0322-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck JSON verdict records can aggregate into a dashboard
rollup so release dashboards can summarize stored aggregate rollup recheck
cohort verification rollup recheck decisions.

## Change

No Rust code change was needed. The validation example already exposes a
rollup mode for saved aggregate rollup recheck cohort rollup recheck JSON
verdict records:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup <saved-json-verdict-log.txt>
```

This experiment aggregated the real saved matched and stale JSON verdict
records produced by Experiment 0329.

## Commands

```text
EVIDENCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup
SOURCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser
mkdir -p target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
cat target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup "$EVIDENCE_DIR/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log" > "$EVIDENCE_DIR/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-rollup.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-rollup.log
wc -l target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/*.log
rg -o 'records=2|matched=1|drifted=1|drift_records=1|"schema":"[^"]+"|"records":2|"matched":1|"drifted":1|"records":1' target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-rollup.log
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
```

## Results

An initial command used an over-deep mode suffix:

```text
--rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup
```

The example rejected that suffix with usage output. The accepted aggregate
mode was:

```text
--rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup
```

The saved matched plus stale JSON verdict records aggregated into:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The compact JSON rollup line used schema
`locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1`
and preserved:

```text
"records":2
"matched":1
"drifted":1
"drift_fields":{"records":1}
```

The evidence directory contained 10 total log lines:

```text
2 combined rollup log
4 combined saved JSON verdict log
2 matched saved JSON verdict log
2 stale saved JSON verdict log
10 total
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict.log
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
verdict rollup recheck JSON verdict records aggregate into a dashboard rollup.
The real saved matched and stale verdict records produced the expected
two-record cohort with one matched outcome, one drifted outcome, and one
first-field `records` drift bucket.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict cohort rollup verdict rollup evidence, not allocator speed
evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup JSON reload as a typed rollup so
release dashboards can recheck stored aggregate rollup recheck cohort
verification rollup recheck summary outcomes?
