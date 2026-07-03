# Experiment 0262: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0254-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary.md`

The postulate said that saved verdict rollup verification JSON records could be
aggregated into a compact typed dashboard summary of matched and drifted
rollup-verification artifacts.

## Change

`locus-validate` now exports:

```text
summarize_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_json_log
format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_json_line
```

The summary counts verifier JSON records, matched artifacts, drifted artifacts,
and first-drift buckets for verifier-rollup counters. The validation example
now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary <saved-verdict-rollup-verification-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/matched-verdict-rollup-verification.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift-json-parser/drifted-record-verdict-rollup-verification.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary/combined-verdict-rollup-verification-summary.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- mixed matched and drifted verifier JSON records aggregate into two records;
- the summary buckets `records` drift from a stale verifier-rollup artifact;
- logs with no verifier JSON records are rejected;
- internally inconsistent verifier JSON records are rejected through the typed
  parser.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real combined verifier log summarized into:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The compact JSON line reported:

```text
schema=locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary.v1 records=2 matched=1 drifted=1 drift_fields.records=1
```

`cargo clippy --workspace --all-targets -- -D warnings` initially rejected the
first accumulator shape because the function exceeded the `too_many_lines`
limit. The final version splits drift-field bucket selection into a small
helper and passes clippy without suppressing the lint.

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
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Dashboard archives can now summarize saved verdict rollup verification JSON
artifacts without reading human-readable lines. The real archive with one
matched verifier artifact and one controlled stale `records=1` verifier
artifact reports two records, one match, one drift, and one `records` drift
bucket.

## Next Question

Can verdict rollup verification summary JSON be parsed back into a typed
summary so dashboard archives can validate their aggregate verifier summaries?
