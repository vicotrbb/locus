# Experiment 0162: THP Run Summary Policy Candidate

Date: 2026-07-03

## Postulate

[Postulate 0154](../postulates/0154-thp-run-summary-policy-candidate.md)
claimed that the repeated THP benchmark evidence summary should emit an
explicit policy-candidate verdict so benchmark timing ranges cannot be mistaken
for allocator policy support when page-size evidence is mixed, unavailable, or
still shows base pages.

## Change

Added `MappedScratchThpBenchmarkEvidenceRunPolicyCandidateReason` with these
machine-readable reasons:

- `ready`;
- `unavailable_reports`;
- `mixed_page_evidence`;
- `no_hugepage_adoption`.

`MappedScratchThpBenchmarkEvidenceRunSummary` now prints:

```text
policy_candidate=<bool> policy_candidate_reason=<reason>
```

The verdict is intentionally strict. A run is a policy candidate only when every
compact report is ready, the page-size cohort is consistent, and every report
observed hugepage adoption.

## Validation

Host commands:

```bash
cargo test -p locus-validate thp_benchmark_evidence_run_summary
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc 'set -eu; for run in 1 2; do /usr/local/cargo/bin/cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 1 > /tmp/locus-thp-bench-$run.out 2>&1; /usr/local/cargo/bin/cargo run -q -p locus-validate --example mapped_scratch_thp_benchmark_evidence_report -- /tmp/locus-thp-bench-$run.out > /tmp/locus-thp-report-$run.out; cat /tmp/locus-thp-report-$run.out; done; /usr/local/cargo/bin/cargo run -q -p locus-validate --example mapped_scratch_thp_benchmark_evidence_run_summary -- /tmp/locus-thp-report-1.out /tmp/locus-thp-report-2.out | tee /tmp/locus-thp-summary.out; grep -q "mapped_scratch_thp_benchmark_evidence_runs=ready" /tmp/locus-thp-summary.out; grep -q "reports=2" /tmp/locus-thp-summary.out; grep -q "page_evidence_cohort=consistent" /tmp/locus-thp-summary.out; grep -q "policy_candidate=false" /tmp/locus-thp-summary.out; grep -q "policy_candidate_reason=no_hugepage_adoption" /tmp/locus-thp-summary.out'
```

Host results:

- `cargo test -p locus-validate thp_benchmark_evidence_run_summary`: passed,
  6 focused tests.
- `cargo test --workspace`: passed, 191 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Docker report outputs:

```text
mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false default_time_lower_ps=785430000 default_time_estimate_ps=814360000 default_time_upper_ps=838200000 hugepage_time_lower_ps=32536000 hugepage_time_estimate_ps=32949000 hugepage_time_upper_ps=33609000 no_hugepage_time_lower_ps=784790000 no_hugepage_time_estimate_ps=800420000 no_hugepage_time_upper_ps=834130000
mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false default_time_lower_ps=772910000 default_time_estimate_ps=780770000 default_time_upper_ps=792040000 hugepage_time_lower_ps=31270000 hugepage_time_estimate_ps=31685000 hugepage_time_upper_ps=32431000 no_hugepage_time_lower_ps=779550000 no_hugepage_time_estimate_ps=787280000 no_hugepage_time_upper_ps=798450000
```

Docker summary output:

```text
mapped_scratch_thp_benchmark_evidence_runs=ready reports=2 ready_reports=2 unavailable_reports=0 hugepage_adoption_reports=0 base_page_reports=2 major_fault_reports=0 default_time_estimate_min_ps=780770000 default_time_estimate_max_ps=814360000 hugepage_time_estimate_min_ps=31685000 hugepage_time_estimate_max_ps=32949000 no_hugepage_time_estimate_min_ps=787280000 no_hugepage_time_estimate_max_ps=800420000 hugepage_vs_default_time_estimate_min_delta_ps=-781411000 hugepage_vs_default_time_estimate_max_delta_ps=-749085000 page_evidence_cohort=consistent policy_candidate=false policy_candidate_reason=no_hugepage_adoption cohort_hugepage_observed=no cohort_hugepage_source=smaps cohort_hugepage_kernel_page_kb=4
```

## Interpretation

The postulate survived.

The Docker allocation benchmark still shows a large timing difference for
hugepage advice, but both real report lines observed base pages. The summary now
prints `policy_candidate=false` and
`policy_candidate_reason=no_hugepage_adoption`, which is the correct
interpretation boundary for this environment.

This keeps the timing evidence useful while preventing the summary line from
being used as support for THP-aware allocator policy.

## Next Step

Keep the policy-candidate verdict in all repeated THP result summaries. The next
THP policy experiment should start only after a run has
`policy_candidate=true`.
