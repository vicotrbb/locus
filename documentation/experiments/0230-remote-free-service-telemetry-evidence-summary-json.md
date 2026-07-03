# Experiment 0230: Remote-Free Service Telemetry Evidence Summary JSON

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0222-remote-free-service-telemetry-evidence-summary-json.md`

The postulate said that the remote-free service telemetry collector could emit
a compact JSON summary beside `manifest.txt` and `validation-summary.txt` that
records collection mode, run count, benchmark filters or saved-output inputs,
Criterion arguments, and artifact byte counts without weakening the existing
manifest-backed validation path.

## Change

Added `collection-summary.json` to
`remote_free_service_telemetry_collect` evidence bundles.

The JSON summary uses schema
`locus.remote_free_service.telemetry.collection_summary.v1` and records:

- `collection_mode`;
- `run_id`;
- `output_count`;
- `criterion_args`;
- `sources` with role, label, input, and output artifact path;
- `artifacts` with kind, relative path, role for output files, and byte count.

The collector writes the JSON summary after captured outputs,
`manifest.txt`, and `validation-summary.txt` exist, so byte counts come from
filesystem metadata rather than in-memory string lengths.

## Commands

```text
cargo test -p locus-validate --example remote_free_service_telemetry_collect -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_collect -- --run-id apply-confirm-summary-1783084007-13676 --bench --repeat 3 target/locus-evidence/remote-free-service-summary-json apply-confirm-summary remote_free_service_runtime_apply_confirm -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
sed -n '1,220p' target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
sed -n '1,80p' target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/validation-summary.txt
wc -c target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/apply-confirm-summary-01.txt target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/apply-confirm-summary-02.txt target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/apply-confirm-summary-03.txt target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/manifest.txt target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/validation-summary.txt target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
sed -n '1,40p' target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/manifest.txt
```

## Results

The focused collector-summary test passed:

```text
test tests::writes_collection_summary_json_with_artifact_byte_counts ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real repeated direct capture created `collection-summary.json` and printed
its path:

```text
remote_free_service_telemetry_evidence_collection mode=benchmark_capture directory=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676 manifest=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/manifest.txt validation_summary=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/validation-summary.txt collection_summary=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json outputs=3
```

The generated validation summary was:

```text
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-01 candidate_runs=2 accepted_runs=2 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=3 min_estimate_ps=53611000 max_estimate_ps=56031000 spread_ps=2420000
```

The generated manifest was:

```text
# role label path
baseline apply-confirm-summary-01 apply-confirm-summary-01.txt
candidate apply-confirm-summary-02 apply-confirm-summary-02.txt
candidate apply-confirm-summary-03 apply-confirm-summary-03.txt
```

The JSON summary recorded:

```text
"schema": "locus.remote_free_service.telemetry.collection_summary.v1"
"collection_mode": "benchmark_capture"
"run_id": "apply-confirm-summary-1783084007-13676"
"output_count": 3
"criterion_args": ["--sample-size", "10", "--warm-up-time", "0.1", "--measurement-time", "0.1"]
```

It also recorded the three output sources, all using
`remote_free_service_runtime_apply_confirm` as the benchmark filter.

The JSON artifact byte counts matched `wc -c`:

```text
3275 apply-confirm-summary-01.txt
3178 apply-confirm-summary-02.txt
3260 apply-confirm-summary-03.txt
209 manifest.txt
330 validation-summary.txt
1545 collection-summary.json
```

The byte counts recorded inside `collection-summary.json` were:

```text
3275 apply-confirm-summary-01.txt
3178 apply-confirm-summary-02.txt
3260 apply-confirm-summary-03.txt
209 manifest.txt
330 validation-summary.txt
```

The summary intentionally does not include its own byte count because it is
written after the JSON value is assembled.

## Interpretation

The postulate survived.

The collector now produces a compact index file for each evidence bundle while
preserving the existing validation contract. Downstream tooling can inspect
run count, mode, arguments, source labels, artifact names, and byte counts
without scanning captured Criterion output files.

## Next Question

Can a validation command read `collection-summary.json`, verify that all listed
artifact byte counts still match the filesystem, and then run the existing
manifest-backed stability check from that summary path?
