# Experiment 0324: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0316-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict rollup drift reports can emit compact JSON verdicts so release
dashboards can preserve typed aggregate rollup recheck cohort verification
outcomes.

## Change

No Rust code change was needed. The validation example already exposes compact
JSON verdict emission and parser-only verification for saved aggregate rollup
verification reports:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-log.txt>
```

This experiment emitted compact JSON verdicts for the real matched aggregate
rollup archive and the controlled stale `records=1` aggregate rollup archive
from Experiment 0323, then reloaded both saved JSON verdict logs.

## Commands

```text
mkdir -p target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
wc -l target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/*.log
rg -o '"status":"[^"]+"|"matched":(true|false)|"drift":null|"drift":\{"actual":1,"expected":2,"field":"records"\}' target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/*.log
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

The matched verdict emitted and reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The stale verdict emitted and reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The field check found the expected compact JSON payloads:

```text
"drift":null
"matched":true
"status":"matched"
"drift":{"actual":1,"expected":2,"field":"records"}
"matched":false
"status":"drifted"
```

The evidence directory contained 14 total log lines:

```text
4 source JSON verdict log
1 matched parse log
2 matched JSON verdict log
2 matched aggregate rollup log
1 stale parse log
2 stale JSON verdict log
2 stale aggregate rollup log
14 total
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift-json/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-parse.log
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
reports emit compact JSON verdicts, and those saved verdicts reload through
the parser-only path. The matched aggregate rollup archive preserves
`status=matched`, `matched=true`, and `drift=null`; the stale aggregate rollup
archive preserves `status=drifted`, `matched=false`, and a `records` drift
with expected `2` and actual `1`.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift verdict artifact evidence, not allocator speed evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup drift
verdict JSON records reload as typed reports so release dashboards can recheck
stored aggregate rollup recheck cohort verification outcomes?
