# Experiment 0260: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Drift JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0252-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-drift-json.md`

The postulate said that verdict rollup drift verification could emit
structured JSON verdicts so dashboard jobs can archive matched and drifted
rollup checks without parsing stderr.

## Change

`locus-validate` now exports:

```text
check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_line
```

It also exports:

```text
RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupDrift
RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollupVerification
```

The report builder recomputes a verdict rollup from saved verdict JSON records,
parses an archived verdict rollup JSON line, records the first drift counter,
and returns a typed report for both matched and drifted cases. The formatter
emits a compact JSON line with schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification.v1
```

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json <saved-verdict-log.txt> <saved-verdict-rollup-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/combined-verdict-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/combined-verdict-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/matched-verdict-rollup-verification.log
perl -pe 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/combined-verdict-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/drifted-record-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/drifted-record-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json/drifted-record-verdict-rollup-verification.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- matched archived verdict rollup verification emits `status=matched`,
  `matched=true`, `drift=null`, and expected plus actual rollups;
- a stale archived `records` counter emits `status=drifted`,
  `matched=false`, and `drift.field=records`;
- strict verification still rejects the same stale archived rollup with
  `CountDrift`.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real matched verdict rollup verification emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification=matched records=2 matched=1 drifted=1 drift_records=1 drift_rollup_hosts_present=0 drift_rollup_hosts_missing=0 drift_bundle_hosts=0 drift_bundle_hosts_missing=0 drift_status_valid_bundles=0 drift_status_drifted_summaries=0 drift_status_missing_artifacts=0 drift_status_other_failures=0
{"actual":{"drift_bundle_hosts":0,"drift_bundle_hosts_missing":0,"drift_fields":{"bundle_hosts":0,"bundle_hosts_missing":0,"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":0,"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":0},"drift_records":1,"drift_rollup_hosts_missing":0,"drift_rollup_hosts_present":0,"drift_status_drifted_summaries":0,"drift_status_missing_artifacts":0,"drift_status_other_failures":0,"drift_status_valid_bundles":0,"drifted":1,"matched":1,"records":2,"status_coverage":{"drifted":1,"matched":1}},"drift":null,"expected":{"drift_bundle_hosts":0,"drift_bundle_hosts_missing":0,"drift_fields":{"bundle_hosts":0,"bundle_hosts_missing":0,"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":0,"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":0},"drift_records":1,"drift_rollup_hosts_missing":0,"drift_rollup_hosts_present":0,"drift_status_drifted_summaries":0,"drift_status_missing_artifacts":0,"drift_status_other_failures":0,"drift_status_valid_bundles":0,"drifted":1,"matched":1,"records":2,"status_coverage":{"drifted":1,"matched":1}},"matched":true,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification.v1","status":"matched"}
```

The controlled stale `records=1` archived rollup emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
{"actual":{"drift_bundle_hosts":0,"drift_bundle_hosts_missing":0,"drift_fields":{"bundle_hosts":0,"bundle_hosts_missing":0,"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":0,"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":0},"drift_records":1,"drift_rollup_hosts_missing":0,"drift_rollup_hosts_present":0,"drift_status_drifted_summaries":0,"drift_status_missing_artifacts":0,"drift_status_other_failures":0,"drift_status_valid_bundles":0,"drifted":1,"matched":1,"records":1,"status_coverage":{"drifted":1,"matched":1}},"drift":{"actual":1,"expected":2,"field":"records"},"expected":{"drift_bundle_hosts":0,"drift_bundle_hosts_missing":0,"drift_fields":{"bundle_hosts":0,"bundle_hosts_missing":0,"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":0,"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":0},"drift_records":1,"drift_rollup_hosts_missing":0,"drift_rollup_hosts_present":0,"drift_status_drifted_summaries":0,"drift_status_missing_artifacts":0,"drift_status_other_failures":0,"drift_status_valid_bundles":0,"drifted":1,"matched":1,"records":2,"status_coverage":{"drifted":1,"matched":1}},"matched":false,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification.v1","status":"drifted"}
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
test result: ok. 143 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Dashboard jobs can now archive structured matched and drifted verdict rollup
checks without scraping stderr. Strict verification still exists for release
gates, while JSON verdict mode gives a stable artifact for trend dashboards and
post-publication evidence review.

## Next Question

Can verdict rollup verification JSON be parsed back into typed reports so
dashboard archives can recheck their own drift-verdict artifacts?
