# Experiment 0227: Remote-Free Service Telemetry Evidence Collection

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0219-remote-free-service-telemetry-evidence-collection.md`

The postulate said that saved remote-free service telemetry outputs could be
collected into one evidence directory with copied outputs, a manifest, and a
validation summary from a single command without weakening the counter-gated
timing review.

## Change

Added:

- `format_remote_free_service_telemetry_timing_stability_manifest`;
- `RemoteFreeServiceTelemetryTimingStabilityManifestFormatError`;
- `remote_free_service_telemetry_collect` example command.

The collector command accepts an evidence root, one baseline label and saved
output, one or more candidate labels and saved outputs, and an optional
`--run-id`. Without `--run-id`, it creates a run directory named from the
current Unix timestamp. It writes:

- copied output files named `<label>.txt`;
- `manifest.txt`;
- `validation-summary.txt`.

Artifact labels are restricted to ASCII alphanumeric characters plus `.`, `_`,
and `-`, and the exact labels `.` and `..` are rejected.

## Commands

```text
cargo test -p locus-validate remote_free_service -- --nocapture
cargo check -p locus-validate --example remote_free_service_telemetry_collect
cargo run -p locus-validate --example remote_free_service_telemetry_collect -- --run-id apply-confirm-collector-final-1783083181-11538 target/locus-evidence/remote-free-service-collection apply-confirm-a target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt apply-confirm-b target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt apply-confirm-drift target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt
find target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538 -maxdepth 1 -type f -print | sort
sed -n '1,40p' target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/manifest.txt
sed -n '1,80p' target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/validation-summary.txt
wc -c target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/*.txt
```

## Results

The collector command created:

```text
target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/apply-confirm-a.txt
target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/apply-confirm-b.txt
target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/apply-confirm-drift.txt
target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/manifest.txt
target/locus-evidence/remote-free-service-collection/apply-confirm-collector-final-1783083181-11538/validation-summary.txt
```

The generated manifest was:

```text
# role label path
baseline apply-confirm-a apply-confirm-a.txt
candidate apply-confirm-b apply-confirm-b.txt
candidate apply-confirm-drift apply-confirm-drift.txt
```

The generated validation summary was:

```text
remote_free_service_telemetry_timing_stability=mixed baseline=apply-confirm-a candidate_runs=2 accepted_runs=1 discarded_runs=1 timing_ranges=1
remote_free_service_telemetry_timing_discard run=apply-confirm-drift drift_entries=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=56595000 max_estimate_ps=56867000 spread_ps=272000
```

The copied output byte counts matched the saved source outputs:

```text
3298 apply-confirm-a.txt
3274 apply-confirm-b.txt
3274 apply-confirm-drift.txt
163 manifest.txt
404 validation-summary.txt
```

Focused unit tests covered manifest formatting, parser round-trip, duplicate
formatted labels, invalid formatted labels, and the previous parser and
stability cases. The focused remote-free service validator test count
increased from 24 to 27.

## Interpretation

The postulate survived.

The collector created a self-contained evidence directory in one command and
preserved the same counter-gated `mixed` stability result from Experiments
0225 and 0226. The collected validation summary still excluded the controlled
drift output from the timing range.

## Next Question

Can the collector run selected remote-free service telemetry Criterion
benchmarks directly, capture their output into the evidence directory, and
then run the same manifest-backed validation?
