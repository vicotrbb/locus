# Experiment 0225: Remote-Free Service Telemetry Timing Stability

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0217-remote-free-service-telemetry-timing-stability.md`

The postulate said that repeated remote-free service telemetry benchmark
outputs could be summarized by using JSON counter comparison as the admission
gate for timing evidence. Counter-stable candidate outputs should contribute
to timing ranges, while counter-drifted candidates should be reported as
discarded and excluded from timing ranges.

## Change

Added `remote_free_service_timing_stability.rs` in `locus-validate` with:

- `summarize_remote_free_service_telemetry_timing_stability`;
- borrowed run inputs with stable labels;
- typed stability status, report, timing range, discard, and error types.

The `remote_free_service_sample_compare` example keeps its existing pairwise
mode when given one candidate output. When given more than one candidate
output, it prints a repeated-run stability summary, discard lines, and timing
range lines.

## Commands

```text
cargo test -p locus-validate remote_free_service -- --nocapture
cargo run -p locus-validate --example remote_free_service_sample_compare -- target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt
cargo run -p locus-validate --example remote_free_service_sample_compare -- target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt
```

## Results

The existing pairwise command stayed unchanged for the two real saved outputs:

```text
remote_free_service_telemetry_sample_timing_compare=stable baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=0 timing_entries=1
remote_free_service_telemetry_timing_delta benchmark=remote_free_service_runtime_apply_confirm baseline_estimate_ps=56595000 candidate_estimate_ps=56867000 estimate_delta_ps=272000
```

The repeated-run command over baseline A, stable candidate B, and controlled
drift output produced a mixed stability report:

```text
remote_free_service_telemetry_timing_stability=mixed baseline=target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt candidate_runs=2 accepted_runs=1 discarded_runs=1 timing_ranges=1
remote_free_service_telemetry_timing_discard run=target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt drift_entries=1
remote_free_service_telemetry_timing_range benchmark=remote_free_service_runtime_apply_confirm range_runs=2 min_estimate_ps=56595000 max_estimate_ps=56867000 spread_ps=272000
```

Focused unit tests covered accepted candidates, mixed accepted and discarded
candidates, all-drift candidates with no timing ranges, duplicate run labels,
and missing timing intervals for counter-stable candidates. The focused
remote-free service validator test count increased from 12 to 17.

## Interpretation

The postulate survived.

The new summary gives a compact review surface for repeated saved outputs
without weakening the counter gate. The controlled drift candidate was visible
as a discard and did not contribute its timing estimate to the reported range.
The accepted timing range stayed identical to the pairwise estimate delta:
56,595,000 ps to 56,867,000 ps, for a 272,000 ps spread.

## Next Question

Can the repeated-run stability report ingest a directory or manifest of saved
outputs so benchmark evidence collection does not depend on long command
lines?
