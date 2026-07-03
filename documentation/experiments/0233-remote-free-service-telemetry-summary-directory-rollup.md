# Experiment 0233: Remote-Free Service Telemetry Summary Directory Rollup

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0225-remote-free-service-telemetry-summary-directory-rollup.md`

The postulate said that the remote-free service telemetry summary validator
could validate every `collection-summary.json` under an evidence directory and
emit a compact rollup that counts valid bundles, drifted validation summaries,
missing artifacts, other failures, and timing ranges that survived validation.

## Change

Extended `remote_free_service_telemetry_summary_validate` with directory mode:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir <evidence-root>
```

Directory mode recursively discovers `collection-summary.json` files, validates
each bundle through the same single-summary path, and emits:

```text
remote_free_service_telemetry_collection_summary_rollup root=<root> summaries=<n> valid_bundles=<n> drifted_summaries=<n> missing_artifacts=<n> other_failures=<n> timing_ranges=<n>
```

The rollup counts timing ranges only from bundles that fully pass artifact
verification, saved-summary comparison, and manifest-backed stability
validation.

## Commands

```text
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json
```

## Results

The focused example tests passed, including a synthetic directory with one
valid bundle, one drifted saved-summary bundle, and one missing-artifact
bundle:

```text
test tests::rejects_drifted_validation_summary ... ok
test tests::reports_matching_validation_summary ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real evidence directory from Experiment 0230 produced:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
```

## Interpretation

The postulate survived.

The validator can now audit a directory of saved evidence bundles and report
whether the set is clean enough for downstream benchmark review. The first real
rollup found one valid bundle, zero drifted summaries, zero missing artifacts,
zero other failures, and one surviving timing range.

## Next Question

Can the directory rollup persist a small rollup artifact beside the evidence
root so benchmark dashboards or release checks can consume validation status
without rerunning the scanner?
