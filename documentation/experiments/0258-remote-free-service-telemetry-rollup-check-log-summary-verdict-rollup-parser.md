# Experiment 0258: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Parser

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0250-remote-free-service-telemetry-rollup-check-log-summary-verdict-rollup-parser.md`

The postulate said that saved verdict rollup JSON lines could be parsed back
into typed rollups so archived dashboard rollups can be verified after
publication.

## Change

`locus-validate` now exports:

```text
parse_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_json_line
```

The parser checks the verdict rollup JSON schema, reconstructs
`RemoteFreeServiceTelemetryCollectionSummaryRollupCheckLogSummaryVerificationRollup`,
and rejects internally inconsistent records. The rollup JSON now includes flat
drift counters beside the grouped `drift_fields` object, matching the existing
release-check JSON pattern of flat counters plus grouped coverage.

The validation example now accepts:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify <saved-verdict-rollup-log.txt>
```

An intermediate implementation tried to reconstruct drift counters only from
the grouped `drift_fields` object. The focused drift-field mutation test showed
that this could not prove grouped-field drift because there was no independent
counter to compare against. The final implementation emits and parses flat
drift counters, then compares the grouped object against them.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
mkdir -p target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-parser
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup/combined-verdict.log > target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-parser/combined-verdict-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify target/locus-evidence/remote-free-service-rollup-check-log-summary-verdict-rollup-parser/combined-verdict-rollup.log
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Results

Focused collection-summary tests passed:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The new tests prove:

- formatting then parsing a verdict rollup round-trips to the same typed
  rollup;
- schema drift is rejected;
- missing grouped fields are rejected;
- grouped status coverage drift is rejected;
- grouped drift-field coverage drift is rejected.

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real mixed verdict rollup parsed back into the expected typed rollup:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_rollup_hosts_present=0 drift_rollup_hosts_missing=0 drift_bundle_hosts=0 drift_bundle_hosts_missing=0 drift_status_valid_bundles=0 drift_status_drifted_summaries=0 drift_status_missing_artifacts=0 drift_status_other_failures=0
```

The generated dashboard log carried both human and JSON lines:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_rollup_hosts_present=0 drift_rollup_hosts_missing=0 drift_bundle_hosts=0 drift_bundle_hosts_missing=0 drift_status_valid_bundles=0 drift_status_drifted_summaries=0 drift_status_missing_artifacts=0 drift_status_other_failures=0
{"drift_bundle_hosts":0,"drift_bundle_hosts_missing":0,"drift_fields":{"bundle_hosts":0,"bundle_hosts_missing":0,"records":1,"rollup_hosts_missing":0,"rollup_hosts_present":0,"status_drifted_summaries":0,"status_missing_artifacts":0,"status_other_failures":0,"status_valid_bundles":0},"drift_records":1,"drift_rollup_hosts_missing":0,"drift_rollup_hosts_present":0,"drift_status_drifted_summaries":0,"drift_status_missing_artifacts":0,"drift_status_other_failures":0,"drift_status_valid_bundles":0,"drifted":1,"matched":1,"records":2,"schema":"locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup.v1","status_coverage":{"drifted":1,"matched":1}}
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
test result: ok. 137 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Archived verdict rollup JSON can now be validated as its own dashboard
artifact. The implementation also found and fixed a schema weakness: grouped
drift counters need independent flat counters if the parser is expected to
detect grouped drift-field edits.

## Next Question

Can verdict rollup JSON verification compare archived rollups against a
recomputed rollup from the same saved verdict log?
