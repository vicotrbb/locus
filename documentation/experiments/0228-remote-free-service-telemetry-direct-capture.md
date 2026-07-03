# Experiment 0228: Remote-Free Service Telemetry Direct Capture

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0220-remote-free-service-telemetry-direct-capture.md`

The postulate said that the remote-free service telemetry collector could run
selected Criterion benchmarks directly, capture their output into an evidence
directory, and then reuse the same manifest-backed validation path used for
saved outputs.

## Change

Extended `remote_free_service_telemetry_collect` with opt-in benchmark capture
mode:

```text
--bench <evidence-root> <baseline-label> <baseline-filter> <candidate-label> <candidate-filter> [candidate-label candidate-filter ...] [-- <criterion-arg> ...]
```

In this mode the collector runs:

```text
cargo bench -p locus-alloc --bench remote_free_service_telemetry <filter> [-- <criterion-arg> ...]
```

for each labeled input, sets `LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1`,
captures stdout and stderr, writes each captured output as `<label>.txt`, then
writes the same `manifest.txt` and `validation-summary.txt` artifacts used by
saved-output collection. Failing benchmark commands return an error after the
captured output is written.

Saved-output collection remains the default mode.

## Commands

```text
cargo run -p locus-validate --example remote_free_service_telemetry_collect -- --run-id apply-confirm-direct-1783083385-11799 --bench target/locus-evidence/remote-free-service-direct-capture apply-confirm-direct-a remote_free_service_runtime_apply_confirm apply-confirm-direct-b remote_free_service_runtime_apply_confirm -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
find target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799 -maxdepth 1 -type f -print | sort
sed -n '1,40p' target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/manifest.txt
sed -n '1,80p' target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/validation-summary.txt
rg -n "^\{|time:|remote_free_service_runtime_apply_confirm" target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/apply-confirm-direct-a.txt target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/apply-confirm-direct-b.txt
wc -c target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/*.txt
```

## Results

The collector created:

```text
target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/apply-confirm-direct-a.txt
target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/apply-confirm-direct-b.txt
target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/manifest.txt
target/locus-evidence/remote-free-service-direct-capture/apply-confirm-direct-1783083385-11799/validation-summary.txt
```

The generated manifest was:

```text
# role label path
baseline apply-confirm-direct-a apply-confirm-direct-a.txt
candidate apply-confirm-direct-b apply-confirm-direct-b.txt
```

The generated validation summary was:

```text
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-direct-a candidate_runs=1 accepted_runs=1 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=56591000 max_estimate_ps=56765000 spread_ps=174000
```

Both captured output files contained JSON telemetry rows and Criterion timing
intervals for `remote_free_service_runtime_apply_confirm`. The captured point
estimates were:

```text
apply-confirm-direct-a: 56.765 us
apply-confirm-direct-b: 56.591 us
```

The persisted artifact byte counts were:

```text
3187 apply-confirm-direct-a.txt
3187 apply-confirm-direct-b.txt
137 manifest.txt
327 validation-summary.txt
```

## Interpretation

The postulate survived.

Direct capture now removes the manual benchmark-output capture step while
preserving the same evidence contract: captured outputs are durable files,
`manifest.txt` names them, and `validation-summary.txt` is produced by the
same counter-gated stability report as saved-output collection.

## Next Question

Can the direct collector run a small repeated-capture cohort for one benchmark
label and automatically name each repeated run while preserving the same
manifest-backed validation?
