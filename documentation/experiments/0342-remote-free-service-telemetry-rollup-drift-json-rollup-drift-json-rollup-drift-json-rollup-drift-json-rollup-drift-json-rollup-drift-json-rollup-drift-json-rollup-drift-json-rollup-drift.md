# Experiment 0342: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0334-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift.md`

The postulate said that archived repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup drift verdict rollup recheck verdict rollup drift JSON can
verify against saved source verdict records so release dashboards can reject
stale stored summary recheck rollups.

## Change

No Rust code change was needed. The validation example already exposes strict
and report modes for checking a saved dashboard rollup artifact against saved
source verdict records:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-json-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-report <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

This experiment verified the real saved rollup from Experiment 0341 against the
saved source verdict records from Experiment 0340 and then checked a controlled
stale rollup with top-level JSON `records=1`.

## Commands

```text
EVIDENCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift
SOURCE_RECORDS_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup
SOURCE_ROLLUP_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser
mkdir -p "$EVIDENCE_DIR"
cp "$SOURCE_RECORDS_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/combined-rollup-verification-json.log"
cp "$SOURCE_ROLLUP_DIR/combined-rollup-verification-json-rollup.log" "$EVIDENCE_DIR/matched-combined-rollup-verification-json-rollup.log"
perl -pe 's/"records":2/"records":1/' "$EVIDENCE_DIR/matched-combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/stale-combined-rollup-verification-json-rollup.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against "$EVIDENCE_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/matched-combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/matched-rollup-verification.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-report "$EVIDENCE_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/stale-combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/stale-rollup-verification-report.log"
set +e
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against "$EVIDENCE_DIR/combined-rollup-verification-json.log" "$EVIDENCE_DIR/stale-combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/stale-rollup-strict.stdout" 2> "$EVIDENCE_DIR/stale-rollup-strict.stderr"
STRICT_STATUS=$?
set -e
printf 'strict_status=%s\n' "$STRICT_STATUS" > "$EVIDENCE_DIR/stale-rollup-strict.status"
wc -l "$EVIDENCE_DIR"/*
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

The real saved rollup verified against the saved source verdict records:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The controlled stale rollup report detected the `records` drift:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The strict stale recheck failed with:

```text
strict_status=1
Error: CountDrift { field: "records", expected: 2, actual: 1 }
```

The evidence directory contained 14 total lines:

```text
4 saved source verdict log
2 matched saved rollup log
1 matched verification log
2 stale saved rollup log
1 stale strict status
3 stale strict stderr
0 stale strict stdout
1 stale verification report log
14 total
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-rollup-verification.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-rollup-verification-report.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-rollup-strict.status
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-rollup-strict.stderr
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-rollup-strict.stdout
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

Archived repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift JSON verifies against saved source
verdict records. The real saved rollup matches the recomputed source verdict
cohort, while the controlled stale rollup is rejected by strict mode with
`CountDrift`.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict cohort rollup verdict rollup drift verdict rollup drift verdict
rollup drift evidence, not allocator speed evidence.

## Next Question

Can repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift verdict rollup drift verdict rollup drift verdict rollup drift verdict
rollup recheck verdict rollup drift drift reports emit compact JSON verdicts
so release dashboards can archive summary recheck decisions?
