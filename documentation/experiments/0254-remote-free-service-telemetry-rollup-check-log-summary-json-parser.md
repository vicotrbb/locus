# Experiment 0254: Remote-Free Service Telemetry Rollup Check Log Summary JSON Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0246-remote-free-service-telemetry-rollup-check-log-summary-json-parser.md`

The postulate said that saved-log summary JSON lines could be parsed back into
typed `RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummary`
reports to verify archived dashboard artifacts without rescanning the original
release-check records.

## Change

`locus-validate` now exports
`parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_line`.

The parser accepts schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v1
```

It reconstructs the typed saved-log summary from flat JSON fields and verifies
that grouped `host_coverage` and `status_coverage` fields agree with those
flat counters.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verify <saved-log.txt>
```

The command scans an archived summary log, finds the JSON summary line, parses
it through the typed parser, and prints the canonical human-readable summary.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/host-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/no-host-rollup-check.log
cat target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/host-rollup-check.log target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/no-host-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/combined-rollup-check.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/combined-rollup-check.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/combined-rollup-check-summary.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-json-parser/combined-rollup-check-summary.log
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
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The parser tests prove:

- formatting then parsing a saved-log summary JSON line round-trips to the
  same typed summary;
- schema drift is rejected;
- missing grouped fields are rejected;
- grouped host coverage drift is rejected;
- grouped status coverage drift is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real archived summary log contained the human line and the JSON summary
line:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log=ok records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
{"bundle_hosts":1,"bundle_hosts_missing":1,"host_coverage":{"bundle_hosts":1,"bundle_hosts_missing":1,"rollup_hosts_missing":0,"rollup_hosts_present":2},"records":2,"rollup_hosts_missing":0,"rollup_hosts_present":2,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log.v1","status_coverage":{"drifted_summaries":0,"missing_artifacts":0,"other_failures":0,"valid_bundles":2},"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":2}
```

Parsing the saved JSON summary line back into the typed summary printed:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log=ok records=2 rollup_hosts_present=2 rollup_hosts_missing=0 bundle_hosts=1 bundle_hosts_missing=1 status_valid_bundles=2 status_drifted_summaries=0 status_missing_artifacts=0 status_other_failures=0
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
test result: ok. 121 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Archived dashboard summary JSON can now be checked independently for schema,
field types, and internal grouped-counter consistency. The verification path
does not reread rollup artifacts or the original per-artifact release-check
records.

## Next Question

Can saved-log summary JSON verification report drift against a separately
recomputed saved-log summary from the same CI log?
