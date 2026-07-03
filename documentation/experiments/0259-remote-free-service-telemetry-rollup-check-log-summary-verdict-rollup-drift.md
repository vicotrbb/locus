# Experiment 0259: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0251-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-drift.md`

The postulate said that archived verdict rollup JSON could be compared against
a recomputed rollup from the same saved verdict log, catching stale dashboard
rollups after publication.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log
verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_log
```

The verifier recomputes a typed verdict rollup from saved verdict JSON records,
parses the archived verdict rollup JSON log, and compares every counter in a
stable order. It returns the archived typed rollup on match and returns
`CountDrift` for the first counter mismatch.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against <saved-verdict-log.txt> <saved-verdict-rollup-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift/combined-verdict-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift/combined-verdict-rollup.log
perl -pe 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift/combined-verdict-rollup.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift/drifted-record-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-drift/drifted-record-rollup.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- a freshly generated archived verdict rollup matches the recomputed source
  verdict log;
- the archived rollup JSON log parser finds the saved JSON line in a mixed log;
- a stale archived `records` counter is rejected;
- grouped status coverage drift is rejected;
- grouped drift-field coverage drift is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real archived rollup matched the real mixed verdict log:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_rollup_hosts_present=0 drift_rollup_hosts_missing=0 drift_bundle_hosts=0 drift_bundle_hosts_missing=0 drift_status_valid_bundles=0 drift_status_drifted_summaries=0 drift_status_missing_artifacts=0 drift_status_other_failures=0
```

A controlled stale edit from JSON `"records":2` to `"records":1` failed with:

```text
Error: CountDrift { field: "records", expected: 2, actual: 1 }
drift_exit_code=1
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
test result: ok. 141 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Archived verdict rollups now have both internal consistency checks and
source-log drift checks. A dashboard can validate a published rollup against
the saved verdict records it summarizes, and a stale but internally valid
counter edit is caught.

## Next Question

Can verdict rollup drift verification emit a structured JSON verdict so
dashboard jobs can archive matched and drifted rollup checks without parsing
stderr?
