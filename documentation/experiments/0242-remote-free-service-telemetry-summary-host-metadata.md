# Experiment 0242: Remote-Free Service Telemetry Summary Host Metadata

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0234-remote-free-service-telemetry-summary-host-metadata.md`

The postulate said that remote-free service telemetry collection summaries
could record direct-capture host metadata while staying compatible with
existing schema v1 validation.

## Change

Added optional host metadata to parsed collection summaries:

```text
RemoteFreeServiceTelemetryCollectionSummaryHost
```

The collector now writes a `host` object into `collection-summary.json` with:

- Rust target operating system;
- Rust target CPU architecture;
- hostname when `HOSTNAME` or `COMPUTERNAME` is visible to the process.

The parser accepts summaries without `host`, summaries with a string hostname,
and summaries with `hostname: null`. Artifact validation remains tied to the
listed artifact paths and byte counts.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_collect -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_collect -- --run-id apply-confirm-summary-host-1783084007 --bench --repeat 2 target/locus-evidence/remote-free-service-summary-host-json apply-confirm-summary-host remote_free_service_runtime_apply_confirm -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json
sed -n '1,220p' target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json
wc -c target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/apply-confirm-summary-host-01.txt target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/apply-confirm-summary-host-02.txt target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/manifest.txt target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/validation-summary.txt target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-host-json --write-rollup
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The first real validator run exposed a bug in the initial parser patch:

```text
Error: InvalidFieldType("hostname")
```

The collector correctly wrote `hostname: null` because this process did not
expose `HOSTNAME` or `COMPUTERNAME`. The parser initially accepted absent
hostname but rejected explicit JSON null. The parser was fixed to treat null
hostname as no hostname only for host metadata.

The focused parser tests passed after the fix:

```text
test remote_free_service_collection_summary::tests::parses_collection_summary ... ok
test remote_free_service_collection_summary::tests::parses_collection_summary_host_metadata ... ok
test remote_free_service_collection_summary::tests::parses_collection_summary_host_metadata_without_hostname ... ok
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The collector test passed with host metadata assertions:

```text
test tests::writes_collection_summary_json_with_artifact_byte_counts ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real direct capture wrote a metadata-bearing summary:

```text
remote_free_service_telemetry_evidence_collection mode=benchmark_capture directory=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007 manifest=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/manifest.txt validation_summary=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/validation-summary.txt collection_summary=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json outputs=2
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-host-01 candidate_runs=1 accepted_runs=1 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=53099000 max_estimate_ps=56033000 spread_ps=2934000
```

The real summary recorded:

```text
"host": {
  "arch": "aarch64",
  "hostname": null,
  "os": "macos"
}
```

The validator accepted the real summary:

```text
remote_free_service_telemetry_collection_summary_validation=ok summary=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json manifest=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/manifest.txt collection_mode=benchmark_capture run_id=apply-confirm-summary-host-1783084007 output_count=2
remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts=4 verified_bytes=6937
remote_free_service_telemetry_validation_summary=matched path=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/validation-summary.txt bytes=335
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-host-01 candidate_runs=1 accepted_runs=1 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=53099000 max_estimate_ps=56033000 spread_ps=2934000
```

The generated artifact byte counts were:

```text
3258 apply-confirm-summary-host-01.txt
3179 apply-confirm-summary-host-02.txt
165 manifest.txt
335 validation-summary.txt
1327 collection-summary.json
```

The new evidence root also rolled up cleanly:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-host-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json bytes=594
```

The broad gates passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived after refinement.

Per-bundle collection summaries now preserve capture host context even if a
rollup is regenerated elsewhere. The validation path remains compatible with
older schema v1 summaries and still checks artifact integrity through the
listed files and byte counts.

## Next Question

Can the summary validator print host metadata in its one-line success output
without making host metadata mandatory for old evidence bundles?
