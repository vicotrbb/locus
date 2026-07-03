# Experiment 0231: Remote-Free Service Telemetry Summary Validation

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0223-remote-free-service-telemetry-summary-validation.md`

The postulate said that a validation command could use
`collection-summary.json` as the entrypoint for a remote-free service telemetry
evidence bundle, verify every listed artifact byte count against the
filesystem, and then run the existing manifest-backed timing stability check
without changing the underlying evidence contract.

## Change

Added a typed parser and artifact verifier for
`collection-summary.json` in `locus-validate`.

The parser validates:

- the schema string;
- required string, integer, and array fields;
- output count against listed output artifacts;
- source entries;
- artifact entries.

The verifier rejects absolute artifact paths and paths that escape the summary
directory, then checks each listed byte count against filesystem metadata.

Added the validation command:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- <collection-summary.json>
```

The command verifies the JSON-indexed artifacts, resolves the manifest artifact
from the summary, and then runs the existing manifest-backed timing stability
summary.

## Commands

```text
cargo test -p locus-validate collection_summary -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
```

## Results

The focused collection-summary tests passed:

```text
test remote_free_service_collection_summary::tests::parses_collection_summary ... ok
test remote_free_service_collection_summary::tests::rejects_artifact_byte_count_mismatch ... ok
test remote_free_service_collection_summary::tests::rejects_artifact_path_traversal ... ok
test remote_free_service_collection_summary::tests::rejects_output_count_mismatch ... ok
test remote_free_service_collection_summary::tests::verifies_artifact_byte_counts ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The command validated the real repeated direct-capture bundle from Experiment
0230:

```text
remote_free_service_telemetry_collection_summary_validation=ok summary=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json manifest=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/manifest.txt collection_mode=benchmark_capture run_id=apply-confirm-summary-1783084007-13676 output_count=3
remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts=5 verified_bytes=10252
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-01 candidate_runs=2 accepted_runs=2 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=3 min_estimate_ps=53611000 max_estimate_ps=56031000 spread_ps=2420000
```

The verified artifact byte total is the sum of the five summary-listed
artifacts:

```text
3275 apply-confirm-summary-01.txt
3178 apply-confirm-summary-02.txt
3260 apply-confirm-summary-03.txt
209 manifest.txt
330 validation-summary.txt
```

## Interpretation

The postulate survived.

`collection-summary.json` can now serve as the bundle entrypoint for integrity
checks and still delegates timing stability to the manifest-backed path. The
summary validator also gives the bundle a cheap corruption check before larger
captured Criterion output files are parsed.

## Next Question

Can the summary validator compare `validation-summary.txt` with a freshly
computed stability report and report drift if a saved summary no longer matches
the manifest-backed evidence?
