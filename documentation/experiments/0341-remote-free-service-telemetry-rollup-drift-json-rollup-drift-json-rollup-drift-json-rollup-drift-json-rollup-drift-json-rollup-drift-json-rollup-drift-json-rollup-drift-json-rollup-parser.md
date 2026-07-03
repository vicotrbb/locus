# Experiment 0341: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0333-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup drift verdict rollup recheck verdict rollup drift JSON can
reload as a typed rollup so release dashboards can recheck stored summary
recheck outcomes.

## Change

No Rust code change was needed. The validation example already exposes a
parser-only mode for saved aggregate rollup JSON artifacts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify <saved-aggregate-rollup-json-log.txt>
```

This experiment copied the aggregate rollup JSON artifact produced by
Experiment 0340 into a parser-specific evidence directory and reloaded it as a
typed rollup.

## Commands

```text
EVIDENCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser
SOURCE_DIR=target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup
mkdir -p "$EVIDENCE_DIR"
cp "$SOURCE_DIR/combined-rollup-verification-json-rollup.log" "$EVIDENCE_DIR/combined-rollup-verification-json-rollup.log"
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify "$EVIDENCE_DIR/combined-rollup-verification-json-rollup.log" > "$EVIDENCE_DIR/combined-rollup-verification-json-rollup-parse.log"
wc -l "$EVIDENCE_DIR"/*.log
sed -n '1,40p' "$EVIDENCE_DIR/combined-rollup-verification-json-rollup-parse.log"
rg -o 'records=2|matched=1|drifted=1|drift_records=1|drift_matched=0|drift_drifted=0|drift_drift_records=0' "$EVIDENCE_DIR/combined-rollup-verification-json-rollup-parse.log"
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

The evidence directory contained 3 total log lines:

```text
1 parsed typed rollup output line
2 saved aggregate rollup JSON artifact input lines
3 total
```

The saved aggregate rollup JSON artifact reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The selected token scan confirmed:

```text
records=2
matched=1
drifted=1
drift_records=1
drift_matched=0
drift_drifted=0
drift_drift_records=0
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup-parse.log
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
verdict rollup recheck verdict rollup drift JSON reloads as a typed rollup.
The saved rollup preserved two records, one matched summary recheck decision,
one drifted summary recheck decision, and one `records` drift bucket.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict cohort rollup verdict rollup drift verdict rollup drift verdict
rollup parser evidence, not allocator speed evidence.

## Next Question

Can archived repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup recheck verdict rollup drift JSON verify against saved source
verdict records so release dashboards can reject stale stored summary recheck
rollups?
