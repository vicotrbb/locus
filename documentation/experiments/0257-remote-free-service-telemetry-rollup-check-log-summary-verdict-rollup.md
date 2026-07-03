# Experiment 0257: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0249-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup.md`

The postulate said that saved-log summary verification verdict JSON records
could be aggregated across multiple CI logs into a compact dashboard rollup.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_line
summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_json_log
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line
```

It also exports:

```text
RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup
```

The verifier JSON parser checks schema, status, `matched`, expected summary,
actual summary, and drift fields against the same typed comparison logic used
by the strict verifier. The rollup aggregates verdict count, matched count,
drifted count, and drift-field buckets.

The rollup JSON uses schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup.v1
```

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup <saved-verdict-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/matched-verdict.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify-against-json target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/combined-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-verdict/drifted-record-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/drifted-verdict.log
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/matched-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/drifted-verdict.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log
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
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- matched verdict JSON parses back into the typed report;
- drifted verdict JSON parses back into the typed report;
- a mixed verdict log summarizes matched and drifted counts;
- `records` drift is counted by field;
- the rollup JSON is single-line and schema-tagged.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real combined verdict log used one matched verdict and one controlled
`records=1` drifted verdict. The rollup reported:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_rollup_hosts_present=0 drift_rollup_hosts_missing=0 drift_bundle_hosts=0 drift_bundle_hosts_missing=0 drift_status_valid_bundles=0 drift_status_drifted_summaries=0 drift_status_missing_artifacts=0 drift_status_other_failures=0
{"drift_fields":{"bundle_hosts":0,"bundle_hosts_missing":0,"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":0,"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":0},"drifted":1,"matched":1,"records":2,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup.v1","status_coverage":{"drifted":1,"matched":1}}
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
test result: ok. 132 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Saved dashboard verdicts now have a cohort-level aggregation path. A CI job can
archive individual matched or drifted verdicts and a second job can combine
them into a compact status distribution and drift-field distribution.

## Next Question

Can verdict rollup JSON lines be parsed back into typed rollups so archived
dashboard rollups can be verified after publication?
