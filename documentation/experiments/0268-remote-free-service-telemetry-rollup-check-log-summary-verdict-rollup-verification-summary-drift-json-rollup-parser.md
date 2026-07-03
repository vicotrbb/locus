# Experiment 0268: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0260-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-parser.md`

The postulate said that verifier-summary drift verdict rollup JSON can be
parsed back into typed reports so dashboard archives can recheck cohort-level
aggregate-summary verdict artifacts.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_line
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log
```

The parser validates schema
`locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1`
and reuses the existing flat-plus-grouped verifier-summary counter checks.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify <saved-verifier-summary-verification-rollup-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup/combined-summary-verification-json-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-parser/combined-summary-verification-json-rollup-parse.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- a mixed matched-plus-drifted rollup JSON line parses back into the original
  typed rollup;
- a saved log containing human text plus rollup JSON reloads as the same typed
  rollup;
- grouped `status_coverage` drift is rejected;
- grouped `drift_fields` drift is rejected;
- logs without rollup JSON are rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real mixed verifier-summary drift verdict rollup artifact reloaded as:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

Parsed evidence was saved at:

```text
target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-verification-summary-drift-json-rollup-parser/combined-summary-verification-json-rollup-parse.log
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
test result: ok. 172 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Dashboard archives can now reload verifier-summary drift verdict rollup JSON as
a typed cohort report. The real mixed artifact reloads with two records, one
matched artifact, one drifted artifact, and one `records` drift bucket. The
parser rejects grouped status or drift-field counter inconsistencies.

This is archive-verification evidence, not allocator speed evidence.

## Next Question

Can archived verifier-summary drift verdict rollup JSON be checked against the
saved verifier-summary drift verdict JSON records it summarizes so stale
cohort-level artifacts are rejected?
