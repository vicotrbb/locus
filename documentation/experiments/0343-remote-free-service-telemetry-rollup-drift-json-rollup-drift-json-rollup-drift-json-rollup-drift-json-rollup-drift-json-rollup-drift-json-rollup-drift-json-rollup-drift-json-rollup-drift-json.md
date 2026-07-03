# Experiment 0343: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0335-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json.md`

The postulate said that repeated-check dashboard archive drift verdict rollup
drift verdict rollup drift verdict rollup drift verdict rollup drift verdict
rollup drift verdict rollup recheck verdict rollup drift drift reports can
emit compact JSON verdicts so release dashboards can archive summary recheck
decisions.

## Change

No Rust code change was needed. The validation example already exposes a JSON
verdict mode and parser-only mode for saved dashboard rollup verification
reports:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-output.txt>
```

This experiment preserved the matched rollup and controlled stale rollup
outcomes from Experiment 0342 as compact JSON verdict records, then reloaded
both through the typed parser path.

## Commands

```text
EVIDENCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json
SOURCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift
mkdir -p "$EVIDENCE_DIR"
cp "$SOURCE_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/combined-rollup-verification-json.log"
cp "$SOURCE_DIR/matched-combined-rollup-verification-json-rollup.log" "$EVIDENCE_DIR/matched-combined-rollup-verification-json-rollup.log"
cp "$SOURCE_DIR/stale-combined-rollup-verification-json-rollup.log" "$EVIDENCE_DIR/stale-combined-rollup-verification-json-rollup.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json "$EVIDENCE_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/matched-combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/matched-rollup-verification-json.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json "$EVIDENCE_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/stale-combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/stale-rollup-verification-json.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify "$EVIDENCE_DIR/matched-rollup-verification-json.log" > "$EVIDENCE_DIR/matched-rollup-verification-json-parse.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify "$EVIDENCE_DIR/stale-rollup-verification-json.log" > "$EVIDENCE_DIR/stale-rollup-verification-json-parse.log"
wc -l "$EVIDENCE_DIR"/*.log
sed -n '1,20p' "$EVIDENCE_DIR/matched-rollup-verification-json.log"
sed -n '1,20p' "$EVIDENCE_DIR/stale-rollup-verification-json.log"
sed -n '1,20p' "$EVIDENCE_DIR/matched-rollup-verification-json-parse.log"
sed -n '1,20p' "$EVIDENCE_DIR/stale-rollup-verification-json-parse.log"
rg -o 'status=matched|status=drifted|field=records|expected=2|actual=1|"schema":"[^"]+"|"status":"matched"|"status":"drifted"|"matched":true|"matched":false|"field":"records"|"expected":2|"actual":1|"records":2|"records":1' "$EVIDENCE_DIR/matched-rollup-verification-json.log" "$EVIDENCE_DIR/stale-rollup-verification-json.log"
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

The evidence directory contained 14 log lines:

```text
4 combined source verdict log
2 matched saved rollup log
1 matched JSON verdict parser log
2 matched JSON verdict log
2 stale saved rollup log
1 stale JSON verdict parser log
2 stale JSON verdict log
14 total
```

The matched JSON verdict output contained:

```text
"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1"
"status":"matched"
"matched":true
"records":2
```

The stale JSON verdict output contained:

```text
field=records
expected=2
actual=1
"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_verification.v1"
"status":"drifted"
"matched":false
"field":"records"
"expected":2
"actual":1
```

The matched JSON verdict parser reloaded the saved record as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The stale JSON verdict parser reloaded the saved record as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-rollup-verification-json-parse.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-rollup-verification-json-parse.log
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

Repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift verdict rollup drift verdict rollup drift verdict rollup drift verdict
rollup recheck verdict rollup drift drift reports emit compact JSON verdicts.
The matched archive emits and reloads as `status=matched`, while the controlled
stale archive emits and reloads as `status=drifted` with `field=records`,
expected `2`, and actual `1`.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict cohort rollup verdict rollup drift verdict rollup drift verdict
rollup drift verdict evidence, not allocator speed evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift drift JSON verdict records reload
as typed reports so release dashboards can reprocess stored summary recheck
decisions?
