# Experiment 0255: Remote-Free Service Telemetry Rollup Check Log Summary JSON Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0247-remote-free-service-telemetry-rollup-check-log-summary-json-drift.md`

The postulate said that saved-log summary JSON verification could detect drift
by comparing an archived summary JSON line against a separately recomputed
summary from the same saved rollup-check log.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log
verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_log
```

The parser scans a saved summary log for schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v1
```

The verifier recomputes the saved-log summary from source rollup-check JSON
records, parses the archived summary JSON line, and compares every typed
counter.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verify-against <saved-rollup-check-log.txt> <saved-summary-log.txt>
```

The example entrypoint was also split into helper functions so the additional
mode stays inside the workspace Clippy line-count budget.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/no-host-rollup-check.log
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/host-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/no-host-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check-summary.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify-against target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check-summary.log
sed 's/"records":2/"records":1/' target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check-summary.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/drifted-record-summary.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify-against target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/combined-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-drift/drifted-record-summary.log
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
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The verifier tests prove:

- a recomputed source-log summary and archived summary JSON match for the same
  saved log;
- record-count drift is rejected;
- host-coverage drift is rejected;
- status-coverage drift is rejected;
- a missing archived summary JSON line is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real combined log and archived summary matched:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log=ok records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
```

A controlled edit changed the archived summary JSON from `records=2` to
`records=1` while leaving the JSON internally parseable. The verifier rejected
that edited archive against the recomputed source-log summary:

```text
Error: CountDrift { field: "records", expected: 2, actual: 1 }
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
test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Archived dashboard summary JSON can now be checked against its source CI log,
not only against itself. This closes the gap where an internally consistent
dashboard record could be stale or manually edited after the release-check log
was saved.

## Next Question

Can saved-log summary drift verification emit its own compact JSON verdict for
dashboard ingestion?
