# Experiment 0256: Remote-Free Service Telemetry Rollup Check Log Summary JSON Verdict

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0248-remote-free-service-telemetry-rollup-check-log-summary-json-verdict.md`

The postulate said that saved-log summary drift verification could emit a
compact JSON verdict for both matched and drifted archived summary records.

## Change

`locus-validate` now exports:

```text
check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line
```

It also exports typed verdict data:

```text
RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryDrift
RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerification
```

The report records the recomputed expected summary, the archived actual
summary, and the first counter drift when the summaries differ. The existing
strict verifier still returns `CountDrift` for release gates that should fail
on drift.

The compact JSON verdict uses schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification.v1
```

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verify-against-json <saved-rollup-check-log.txt> <saved-summary-log.txt>
```

That mode prints a human verdict line followed by one compact JSON verdict
line. It returns a drifted JSON verdict for counter drift instead of treating
drift as a process error.

The example command dispatch was split into small mode functions so the
additional dashboard modes stay inside the workspace Clippy line-count budget.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/no-host-rollup-check.log
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/host-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/no-host-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check-summary.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check-summary.log
sed 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/drifted-record-summary.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/drifted-record-summary.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- matching source and archived summary logs produce a matched verdict;
- drifted archived summary logs produce a drifted verdict without losing the
  expected and actual typed summaries;
- the verdict JSON is single-line and schema-tagged;
- grouped `expected`, `actual`, and `drift` fields match the typed verdict.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched archive emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification=matched records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
{"actual":{"bundle_hosts":1,"bundle_hosts_missing":1,"host_coverage":{"bundle_hosts":1,"bundle_hosts_missing":1,"rollup_hosts_missing":0,"rollup_hosts_present":2},"records":2,"rollup_hosts_missing":0,"rollup_hosts_present":2,"status_coverage":{"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"valid_bundles":2},"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":2},"drift":null,"expected":{"bundle_hosts":1,"bundle_hosts_missing":1,"host_coverage":{"bundle_hosts":1,"bundle_hosts_missing":1,"rollup_hosts_missing":0,"rollup_hosts_present":2},"records":2,"rollup_hosts_missing":0,"rollup_hosts_present":2,"status_coverage":{"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"valid_bundles":2},"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":2},"matched":true,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification.v1","status":"matched"}
```

The controlled `records=1` archive emitted a drifted verdict instead of a
process error:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
{"actual":{"bundle_hosts":1,"bundle_hosts_missing":1,"host_coverage":{"bundle_hosts":1,"bundle_hosts_missing":1,"rollup_hosts_missing":0,"rollup_hosts_present":2},"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":2,"status_coverage":{"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"valid_bundles":2},"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":2},"drift":{"actual":1,"expected":2,"field":"records"},"expected":{"bundle_hosts":1,"bundle_hosts_missing":1,"host_coverage":{"bundle_hosts":1,"bundle_hosts_missing":1,"rollup_hosts_missing":0,"rollup_hosts_present":2},"records":2,"rollup_hosts_missing":0,"rollup_hosts_present":2,"status_coverage":{"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"valid_bundles":2},"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":2},"matched":false,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification.v1","status":"drifted"}
```

Final broad gates passed:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

The full workspace suite reported:

```text
test result: ok. 191 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 129 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

The saved-log summary verifier now has two surfaces:

- strict verification for release gates that should fail on drift;
- typed verdict reporting for dashboards that need archived matched or drifted
  records as data.

## Next Question

Can saved-log summary verification verdict JSON records be aggregated across
multiple CI logs into a dashboard rollup?
