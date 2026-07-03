# Experiment 0159: THP Benchmark Timing Report

Date: 2026-07-03

## Postulate

[Postulate 0151](../postulates/0151-thp-benchmark-timing-report.md)
claimed that the mapped scratch THP benchmark evidence report should parse the
three Criterion timing intervals for default, hugepage advice, and no-hugepage
advice from the same benchmark log that provides page-size and fault-counter
evidence.

## Change

Extended `MappedScratchThpBenchmarkEvidenceReport` with parsed Criterion timing
intervals:

- `MappedScratchThpTimingInterval`;
- `MappedScratchThpBenchmarkTimings`;
- `MappedScratchThpBenchmarkTimingsParseError`.

The parser extracts only these benchmark cases:

- `mapped_scratch_write_touch_4mib_default`;
- `mapped_scratch_write_touch_4mib_hugepage_advice`;
- `mapped_scratch_write_touch_4mib_no_hugepage_advice`.

Timing lower bound, point estimate, and upper bound are normalized to integer
picoseconds and printed as `*_time_*_ps` report fields.

## Validation

Host commands:

```bash
cargo test -p locus-validate thp_benchmark_evidence_report
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 1 > /tmp/locus-thp-bench.out 2>&1 && /usr/local/cargo/bin/cargo run -q -p locus-validate --example mapped_scratch_thp_benchmark_evidence_report -- /tmp/locus-thp-bench.out | tee /tmp/locus-thp-report.out && grep -q "mapped_scratch_thp_benchmark_evidence=ready" /tmp/locus-thp-report.out && grep -q "hugepage_observed=no" /tmp/locus-thp-report.out && grep -q "hugepage_adoption=false" /tmp/locus-thp-report.out && grep -q "hugepage_time_estimate_ps=" /tmp/locus-thp-report.out && grep -q "default_time_estimate_ps=" /tmp/locus-thp-report.out'
```

Host results:

- `cargo test -p locus-validate thp_benchmark_evidence_report`: passed, 5
  focused tests.
- `cargo test --workspace`: passed, 181 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Docker report output:

```text
mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false default_time_lower_ps=797120000 default_time_estimate_ps=807680000 default_time_upper_ps=817180000 hugepage_time_lower_ps=31746000 hugepage_time_estimate_ps=32115000 hugepage_time_upper_ps=32924000 no_hugepage_time_lower_ps=797080000 no_hugepage_time_estimate_ps=807860000 no_hugepage_time_upper_ps=818420000
```

## Interpretation

The postulate survived.

The report now binds page-size evidence, fault-counter comparison, and
Criterion timing intervals from one saved benchmark log. The Docker run again
showed fast hugepage-advice timing and lower minor-fault deltas, but the same
report also showed `hugepage_observed=no` and `hugepage_adoption=false`.

This is the strongest current guard against advice-only THP conclusions:
timing, fault deltas, and kernel page-size evidence are now summarized in one
machine-readable line.

## Next Step

Use repeated runs of this report line before changing any THP-related design
guidance. A future repeated-run aggregator should compare only logs with the
same page-size evidence status.
