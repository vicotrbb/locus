# Experiment 0253: Remote-Free Service Telemetry Rollup Check Log Summary JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0245-remote-free-service-telemetry-rollup-check-log-summary-json.md`

The postulate said that saved-log rollup check summaries could emit a compact
JSON line with grouped coverage fields while preserving the existing
human-readable summary line.

## Change

`locus-validate` now exports
`format_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line`.

The JSON line uses schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v1
```

The line includes flat fields for record count, host coverage, and status
coverage, plus grouped `host_coverage` and `status_coverage` objects.

The validation example now prints the existing human line first and the JSON
line second for:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary <saved-log.txt>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json/no-host-rollup-check.log
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-json/host-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json/no-host-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json/combined-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary target/locus-evidence/remote-free-service-rollup-check-log-summary-json/combined-rollup-check.log
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
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The summary test now proves:

- the human-readable summary line stays unchanged;
- the JSON line is a single line;
- the JSON schema is stable;
- flat record, host coverage, and status coverage fields match the typed
  summary;
- grouped `host_coverage` and `status_coverage` values match the same typed
  summary.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real combined log summary preserved the human line and then emitted the
new JSON line:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log=ok records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
{"bundle_hosts":1,"bundle_hosts_missing":1,"host_coverage":{"bundle_hosts":1,"bundle_hosts_missing":1,"rollup_hosts_missing":0,"rollup_hosts_present":2},"records":2,"rollup_hosts_missing":0,"rollup_hosts_present":2,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v1","status_coverage":{"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"valid_bundles":2},"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":2}
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
test result: ok. 116 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Saved-log summaries now have both stable surfaces:

- the existing first line for human release logs and simple token parsers;
- a schema-tagged JSON line for dashboard ingestion.

The JSON line is derived from the typed saved-log summary, so it does not
create another source of truth.

## Next Question

Can saved-log summary JSON lines be parsed back into typed summaries to verify
dashboard artifacts after CI logs are archived?
