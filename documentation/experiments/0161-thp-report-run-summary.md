# Experiment 0161: THP Report Run Summary

Date: 2026-07-03

## Postulate

[Postulate 0153](../postulates/0153-thp-report-run-summary.md) claimed that
repeated THP benchmark evidence should be summarized from compact report lines
inside `locus-validate` instead of being left as manual spreadsheet work.

## Change

Added repeated compact report-line summarization to `locus-validate`:

- `MappedScratchThpBenchmarkEvidenceRunCohort`;
- `MappedScratchThpBenchmarkEvidenceRunSummary`;
- `MappedScratchThpBenchmarkEvidenceRunSummaryParseError`;
- `summarize_mapped_scratch_thp_benchmark_evidence_report_lines`;
- `mapped_scratch_thp_benchmark_evidence_run_summary` example.

The summary reports ready and unavailable counts, hugepage adoption counts,
base-page counts, major-fault counts, min and max timing estimates, min and max
hugepage-vs-default timing estimate deltas, and whether page-size evidence is a
consistent cohort or mixed.

## Validation

Host commands:

```bash
cargo test -p locus-validate thp_benchmark_evidence
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc 'set -eu; for run in 1 2; do /usr/local/cargo/bin/cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 1 > /tmp/locus-thp-bench-$run.out 2>&1; /usr/local/cargo/bin/cargo run -q -p locus-validate --example mapped_scratch_thp_benchmark_evidence_report -- /tmp/locus-thp-bench-$run.out > /tmp/locus-thp-report-$run.out; cat /tmp/locus-thp-report-$run.out; done; /usr/local/cargo/bin/cargo run -q -p locus-validate --example mapped_scratch_thp_benchmark_evidence_run_summary -- /tmp/locus-thp-report-1.out /tmp/locus-thp-report-2.out | tee /tmp/locus-thp-summary.out; grep -q "mapped_scratch_thp_benchmark_evidence_runs=ready" /tmp/locus-thp-summary.out; grep -q "reports=2" /tmp/locus-thp-summary.out; grep -q "page_evidence_cohort=consistent" /tmp/locus-thp-summary.out'
```

Host results:

- `cargo test -p locus-validate thp_benchmark_evidence`: passed, 13
  focused tests.
- `cargo test --workspace`: passed, 189 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Docker report outputs:

```text
mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false default_time_lower_ps=768740000 default_time_estimate_ps=788080000 default_time_upper_ps=799150000 hugepage_time_lower_ps=32405000 hugepage_time_estimate_ps=32890000 hugepage_time_upper_ps=34029000 no_hugepage_time_lower_ps=775710000 no_hugepage_time_estimate_ps=780090000 no_hugepage_time_upper_ps=785970000
mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false default_time_lower_ps=763110000 default_time_estimate_ps=780470000 default_time_upper_ps=801340000 hugepage_time_lower_ps=32794000 hugepage_time_estimate_ps=33295000 hugepage_time_upper_ps=34168000 no_hugepage_time_lower_ps=771580000 no_hugepage_time_estimate_ps=778590000 no_hugepage_time_upper_ps=782470000
```

Docker summary output:

```text
mapped_scratch_thp_benchmark_evidence_runs=ready reports=2 ready_reports=2 unavailable_reports=0 hugepage_adoption_reports=0 base_page_reports=2 major_fault_reports=0 default_time_estimate_min_ps=780470000 default_time_estimate_max_ps=788080000 hugepage_time_estimate_min_ps=32890000 hugepage_time_estimate_max_ps=33295000 no_hugepage_time_estimate_min_ps=778590000 no_hugepage_time_estimate_max_ps=780090000 hugepage_vs_default_time_estimate_min_delta_ps=-755190000 hugepage_vs_default_time_estimate_max_delta_ps=-747175000 page_evidence_cohort=consistent cohort_hugepage_observed=no cohort_hugepage_source=smaps cohort_hugepage_kernel_page_kb=4
```

## Interpretation

The postulate survived.

The summary accepted two real Docker allocation benchmark report lines and
emitted one machine-readable repeated-run line. The run set remained a
consistent base-page cohort with `hugepage_adoption_reports=0`, so the timing
advantage for hugepage advice is still environment behavior, not proof of huge
page adoption.

The useful result is the evidence boundary: repeated timings can now be compared
only after the page-size cohort is visible in the same line.

## Next Step

Use this summary command for larger repeated THP runs. Do not promote THP policy
guidance until a repeated run set has `page_evidence_cohort=consistent` and
observed hugepage adoption on a Linux host that exposes larger kernel pages.
