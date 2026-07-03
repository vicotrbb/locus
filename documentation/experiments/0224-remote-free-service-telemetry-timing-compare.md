# Experiment 0224: Remote-Free Service Telemetry Timing Compare

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0216-remote-free-service-telemetry-timing-compare.md`

The postulate said that the remote-free service telemetry JSON comparison tool
could parse Criterion timing intervals from the same saved benchmark outputs
and emit a combined report that refuses timing deltas when counter drift is
present.

## Change

Extended `remote_free_service_sample_compare.rs` in `locus-validate` with:

- `parse_remote_free_service_telemetry_timings`;
- `compare_remote_free_service_telemetry_sample_outputs_with_timings`;
- typed timing intervals, timing deltas, combined status, combined report, and
  timing parse errors.

The parser joins timing intervals to benchmark labels already present in the
JSON sample rows. Timing values are normalized to picoseconds. The combined
report emits timing deltas only when the sample comparison is stable. When
counter drift is present, the combined status is `counter_drift` and
`timing_entries=0`.

The `remote_free_service_sample_compare` example now prints the combined
summary, then any sample drift lines, then timing delta lines when allowed.

## Commands

```text
cargo run -p locus-validate --example remote_free_service_sample_compare -- target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt
cargo run -p locus-validate --example remote_free_service_sample_compare -- target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt
cargo fmt --all --check
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
cargo test -p locus-validate remote_free_service -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
```

## Results

The combined tool parsed two real saved JSON-enabled
`remote_free_service_runtime_apply_confirm` outputs, found stable counters,
and emitted one timing delta:

```text
remote_free_service_telemetry_sample_timing_compare=stable baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=0 timing_entries=1
remote_free_service_telemetry_timing_delta benchmark=remote_free_service_runtime_apply_confirm baseline_estimate_ps=56595000 candidate_estimate_ps=56867000 estimate_delta_ps=272000
```

The controlled drift output changed the first JSON `submitted_count` from 768
to 769. The combined tool reported counter drift and emitted no timing delta:

```text
remote_free_service_telemetry_sample_timing_compare=counter_drift baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=1 timing_entries=0
remote_free_service_telemetry_sample_drift benchmark=remote_free_service_runtime_apply_confirm sample=remote_free_service_runtime_apply_confirm_sample field=submitted_count baseline=768 candidate=769
```

Focused unit tests covered JSON row parsing, stable counter comparison, field
drift, missing samples, duplicate sample keys, missing JSON rows, unexpected
schemas, Criterion timing parsing, stable timing delta output, counter-drift
timing suppression, missing timings, and unknown timing units. The focused
test count increased from seven to 12.

## Interpretation

The postulate survived.

The validation command now enforces the intended review order: prove counters
stable first, then inspect normalized timing deltas. The real saved benchmark
outputs produced a timing delta only after the sample comparison passed. The
controlled counter drift suppressed timing output as intended.

## Next Question

Can repeated remote-free service telemetry benchmark runs be summarized into a
small stability report that separates counter-stable timing ranges from runs
that must be discarded due to counter drift?
