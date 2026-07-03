# Experiment 0229: Remote-Free Service Telemetry Repeated Direct Capture

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0221-remote-free-service-telemetry-repeated-direct-capture.md`

The postulate said that the direct remote-free service telemetry collector
could run one selected Criterion benchmark repeatedly, assign deterministic
labels to each run, and preserve the same manifest-backed validation contract
used by saved-output and explicit direct-capture modes.

## Change

Extended `remote_free_service_telemetry_collect` with opt-in repeated benchmark
capture:

```text
--bench --repeat <count> <evidence-root> <base-label> <benchmark-filter> [-- <criterion-arg> ...]
```

The repeated mode generates labels as `<base-label>-01`,
`<base-label>-02`, and so on, runs the same benchmark filter once per label,
writes each captured benchmark output as `<label>.txt`, then emits the same
`manifest.txt` and `validation-summary.txt` artifacts as the existing
collector path.

The label generation rule lives in `locus-validate` so it is shared with the
manifest validation boundary and covered by unit tests.

## Commands

```text
cargo test -p locus-validate repeated_capture -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_collect -- --run-id apply-confirm-repeat-1783083722-12497 --bench --repeat 3 target/locus-evidence/remote-free-service-direct-repeat apply-confirm-repeat remote_free_service_runtime_apply_confirm -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
find target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497 -maxdepth 1 -type f -print | sort
sed -n '1,40p' target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/manifest.txt
sed -n '1,80p' target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/validation-summary.txt
rg -n "^\{|time:|remote_free_service_runtime_apply_confirm" target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/apply-confirm-repeat-01.txt target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/apply-confirm-repeat-02.txt target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/apply-confirm-repeat-03.txt
wc -c target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/*.txt
```

## Results

The repeated-label unit tests passed:

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 86 filtered out
```

The collector created:

```text
target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/apply-confirm-repeat-01.txt
target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/apply-confirm-repeat-02.txt
target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/apply-confirm-repeat-03.txt
target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/manifest.txt
target/locus-evidence/remote-free-service-direct-repeat/apply-confirm-repeat-1783083722-12497/validation-summary.txt
```

The generated manifest was:

```text
# role label path
baseline apply-confirm-repeat-01 apply-confirm-repeat-01.txt
candidate apply-confirm-repeat-02 apply-confirm-repeat-02.txt
candidate apply-confirm-repeat-03 apply-confirm-repeat-03.txt
```

The generated validation summary was:

```text
remote_free_service_telemetry_timing_stability=stable baseline=apply-confirm-repeat-01 candidate_runs=2 accepted_runs=2 discarded_runs=0 timing_ranges=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=3 min_estimate_ps=52900000 max_estimate_ps=53428000 spread_ps=528000
```

All three captured output files contained JSON telemetry rows and Criterion
timing intervals for `remote_free_service_runtime_apply_confirm`. The captured
point estimates were:

```text
apply-confirm-repeat-01: 52.976 us
apply-confirm-repeat-02: 53.428 us
apply-confirm-repeat-03: 52.900 us
```

Each run preserved:

```text
submitted_count=768
drained_count=768
released_bytes=3145728
policy_drains=12
drain_rounds=12
install_count=1
confirm_count=1
rollback_count=0
max_wait_bursts=2
mean_wait_bursts=1.500
final_previous_config_present=false
```

The persisted artifact byte counts were:

```text
3178 apply-confirm-repeat-01.txt
3258 apply-confirm-repeat-02.txt
3258 apply-confirm-repeat-03.txt
203 manifest.txt
328 validation-summary.txt
```

## Interpretation

The postulate survived.

Repeated direct capture now lowers the friction for small stability cohorts:
one benchmark filter and one base label produce a durable manifest-backed
evidence bundle with real Criterion output and counter-gated timing ranges.
The three-run cohort was counter-stable and produced a 528,000 ps timing
spread in the short local validation run.

## Next Question

Can the collector add a compact machine-readable manifest summary that records
run count, benchmark filter, command-line Criterion arguments, and captured
artifact byte counts without parsing the text output files later?
