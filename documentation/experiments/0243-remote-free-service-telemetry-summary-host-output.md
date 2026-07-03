# Experiment 0243: Remote-Free Service Telemetry Summary Host Output

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0235-remote-free-service-telemetry-summary-host-output.md`

The postulate said that the summary validator could print capture host
metadata in its one-line success output without making host metadata mandatory
for older evidence bundles.

## Change

The summary validator now includes host status in the
`remote_free_service_telemetry_collection_summary_validation=ok` line.

For summaries without host metadata it prints:

```text
host_present=false
```

For summaries with host metadata it prints:

```text
host_present=true host_os=<os> host_arch=<arch> host_hostname=<hostname-or-none>
```

The hostname field is tokenized for one-line output, replacing non-token
characters with `_` and using `none` for missing or empty values.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo test -p locus-validate collection_summary -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The example tests passed with host output coverage:

```text
test tests::formats_summary_validation_line_without_host_metadata ... ok
test tests::formats_summary_validation_line_with_host_metadata ... ok
test tests::formats_host_fields_as_single_line_tokens ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The focused collection-summary tests still passed:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

The older no-host bundle still validated and reported `host_present=false`:

```text
remote_free_service_telemetry_collection_summary_validation=ok summary=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/collection-summary.json manifest=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/manifest.txt collection_mode=benchmark_capture run_id=apply-confirm-summary-1783084007-13676 host_present=false output_count=3
remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts=5 verified_bytes=10252
remote_free_service_telemetry_validation_summary=matched path=target/locus-evidence/remote-free-service-summary-json/apply-confirm-summary-1783084007-13676/validation-summary.txt bytes=330
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-01 candidate_runs=2 accepted_runs=2 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=3 min_estimate_ps=53611000 max_estimate_ps=56031000 spread_ps=2420000
```

The host-bearing bundle validated and reported host fields:

```text
remote_free_service_telemetry_collection_summary_validation=ok summary=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/collection-summary.json manifest=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/manifest.txt collection_mode=benchmark_capture run_id=apply-confirm-summary-host-1783084007 host_present=true host_os=macos host_arch=aarch64 host_hostname=none output_count=2
remote_free_service_telemetry_collection_summary_artifacts=verified verified_artifacts=4 verified_bytes=6937
remote_free_service_telemetry_validation_summary=matched path=target/locus-evidence/remote-free-service-summary-host-json/apply-confirm-summary-host-1783084007/validation-summary.txt bytes=335
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-summary-host-01 candidate_runs=1 accepted_runs=1 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=53099000 max_estimate_ps=56033000 spread_ps=2934000
```

The first broad clippy run caught a style issue in the formatter:

```text
called `map(<f>).unwrap_or_else(<g>)` on an `Option` value
```

The formatter was changed to `map_or_else`, and the final broad gates passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived.

Host metadata is now visible in validator logs for new evidence while old
schema v1 bundles remain valid and explicitly report `host_present=false`.
This keeps capture context discoverable without making metadata a required
artifact integrity field.

## Next Question

Can directory rollup bundle rows carry each bundle's capture host metadata
without changing the release-check verdict semantics?
