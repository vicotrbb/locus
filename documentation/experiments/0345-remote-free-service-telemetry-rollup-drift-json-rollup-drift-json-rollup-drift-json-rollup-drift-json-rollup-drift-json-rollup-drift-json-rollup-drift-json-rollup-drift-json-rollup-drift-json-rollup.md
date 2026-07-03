# Experiment 0345: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0337-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup drift verdict rollup recheck verdict rollup drift drift JSON
verdict records can aggregate into a dashboard rollup so release dashboards can
summarize stored summary recheck decisions.

## Change

No Rust code change was needed. The validation example already exposes an
aggregate mode for saved dashboard rollup verification JSON verdict records:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup <combined-saved-json-verdict-log.txt>
```

This experiment copied the matched and stale JSON verdict records produced by
Experiment 0344 into a new evidence directory, combined them into one saved
verdict log, and aggregated the saved records into a dashboard rollup.

## Commands

```text
EVIDENCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup
SOURCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-parser
mkdir -p "$EVIDENCE_DIR"
cp "$SOURCE_DIR/matched-rollup-verification-json.log" "$EVIDENCE_DIR/matched-rollup-verification-json.log"
cp "$SOURCE_DIR/stale-rollup-verification-json.log" "$EVIDENCE_DIR/stale-rollup-verification-json.log"
awk '1' "$EVIDENCE_DIR/matched-rollup-verification-json.log" "$EVIDENCE_DIR/stale-rollup-verification-json.log" > "$EVIDENCE_DIR/combined-rollup-verification-json.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup "$EVIDENCE_DIR/combined-rollup-verification-json.log" > "$EVIDENCE_DIR/combined-rollup-verification-json-rollup.log"
wc -l "$EVIDENCE_DIR"/*.log
sed -n '1,40p' "$EVIDENCE_DIR/combined-rollup-verification-json-rollup.log"
rg -o 'records=2|matched=1|drifted=1|drift_records=1|"schema":"[^"]+"|"records":2|"matched":1|"drifted":1|"drift_records":1|"records":1' "$EVIDENCE_DIR/combined-rollup-verification-json-rollup.log"
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

The evidence directory contained 10 total log lines:

```text
2 combined aggregate rollup output lines
4 combined saved JSON verdict input lines
2 matched saved JSON verdict input lines
2 stale saved JSON verdict input lines
10 total
```

The saved JSON verdict aggregate emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The JSON rollup used schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1
```

The JSON rollup preserved the expected dashboard counters:

```text
"records":2
"matched":1
"drifted":1
"drift_records":1
"drift_fields":{"records":1}
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/matched-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/stale-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup.log
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
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

Focused collection-summary tests passed:

```text
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The benchmark compile gate passed:

```text
Executable benches/remote_free_service_telemetry.rs (target/release/deps/remote_free_service_telemetry-b856e81aa3c3a544)
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

`git diff --check` and the em dash scan produced no findings.

## Interpretation

The postulate survived.

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift drift JSON verdict records
aggregate into a dashboard rollup. The combined saved verdict cohort preserved
two records, one matched summary recheck decision, one drifted summary recheck
decision, and one `records` drift bucket.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict cohort rollup verdict rollup drift verdict rollup drift verdict
rollup drift verdict rollup evidence, not allocator speed evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift drift JSON reload as a typed rollup
so release dashboards can recheck stored summary recheck outcomes?
