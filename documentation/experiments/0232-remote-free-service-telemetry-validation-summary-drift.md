# Experiment 0232: Remote-Free Service Telemetry Validation Summary Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0224-remote-free-service-telemetry-validation-summary-drift.md`

The postulate said that the remote-free service telemetry summary validator
could compare the saved `validation-summary.txt` artifact with a freshly
computed manifest-backed stability report and report drift when the saved
summary no longer matches the evidence bundle.

## Change

Extended `remote_free_service_telemetry_summary_validate` to resolve the
`validation_summary` artifact from `collection-summary.json`, read the saved
summary, recompute the manifest-backed stability report, and compare the two
strings exactly.

The command now prints:

```text
remote_free_service_telemetry_validation_summary=matched path=<path> bytes=<bytes>
```

when the saved summary matches the recomputed report. Drift returns an error
that reports the saved and computed byte counts.

The collection-summary library now also exposes a validation-summary artifact
resolver and reports a specific missing-validation-summary error.

## Commands

```text
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo test -p locus-validate collection_summary -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
```

## Results

The focused command tests passed:

```text
test tests::rejects_drifted_validation_summary ... ok
test tests::reports_matching_validation_summary ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The collection-summary tests now include the validation-summary resolver:

```text
test remote_free_service_collection_summary::reports_missing_validation_summary_artifact ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The real Experiment 0230 bundle validated with a matched saved summary:

```text
remote_free_service_telemetry_collection_summary_validation=ok summary=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json manifest=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/manifest.txt collection_mode=benchmark_capture run_id=apply-confirm-summary-1783084007-13676 output_count=3
remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts=5 verified_bytes=10252
remote_free_service_telemetry_validation_summary=matched path=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/validation-summary.txt bytes=330
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-01 candidate_runs=2 accepted_runs=2 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=3 min_estimate_ps=53611000 max_estimate_ps=56031000 spread_ps=2420000
```

## Interpretation

The postulate survived.

The bundle validator now proves three layers before accepting a saved evidence
bundle:

- the JSON index lists valid, in-bundle artifact paths;
- listed artifact byte counts match the filesystem;
- the saved validation summary exactly matches the freshly computed
  manifest-backed stability report.

This strengthens evidence preservation without changing the collector's
existing output contract.

## Next Question

Can repeated summary validation over a directory of evidence bundles produce a
small rollup that counts valid bundles, drifted summaries, missing artifacts,
and the timing ranges that survived validation?
