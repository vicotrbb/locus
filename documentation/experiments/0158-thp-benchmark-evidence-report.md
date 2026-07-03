# Experiment 0158: THP Benchmark Evidence Report

Date: 2026-07-03

## Postulate

[Postulate 0150](../postulates/0150-thp-benchmark-evidence-report.md)
claimed that Locus should provide a validation report that parses one mapped
scratch THP benchmark log and summarizes page-size evidence, fault-counter
evidence, and whether the hugepage-advice timing can be treated as huge page
adoption evidence.

## Change

Added `crates/locus-validate/src/thp_benchmark_evidence_report.rs` with:

- `MappedScratchThpBenchmarkEvidenceReport`;
- report status and reason enums;
- `parse_mapped_scratch_thp_benchmark_evidence_report_output`.

Added
`crates/locus-validate/examples/mapped_scratch_thp_benchmark_evidence_report.rs`
so saved benchmark output can be classified from the command line.

The report emits one stable line:

```text
mapped_scratch_thp_benchmark_evidence=<status> reason=<reason> page_samples=<status> fault_samples=<status> hugepage_observed=<yes|no|unknown> hugepage_reason=<reason> hugepage_source=<source> hugepage_kernel_page_kb=<value|unknown> hugepage_adoption=<bool> fault_comparison=<status> ...
```

## Validation

Host commands:

```bash
cargo test -p locus-validate thp_benchmark_evidence_report
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 1 > /tmp/locus-thp-bench.out 2>&1 && grep "^thp_page_sample=" /tmp/locus-thp-bench.out && /usr/local/cargo/bin/cargo run -q -p locus-validate --example mapped_scratch_thp_benchmark_evidence_report -- /tmp/locus-thp-bench.out | tee /tmp/locus-thp-report.out && grep -q "mapped_scratch_thp_benchmark_evidence=ready" /tmp/locus-thp-report.out && grep -q "hugepage_observed=no" /tmp/locus-thp-report.out && grep -q "hugepage_adoption=false" /tmp/locus-thp-report.out'
```

Host results:

- `cargo test -p locus-validate thp_benchmark_evidence_report`: passed, 5
  focused tests.
- `cargo test --workspace`: passed, 181 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Docker page samples:

```text
thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
```

Docker report output:

```text
mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false
```

## Interpretation

The postulate survived.

The report correctly classified the real Docker benchmark log as ready
evidence because both page samples and fault samples were available. It also
prevented the dangerous interpretation: despite the hugepage-advice case having
lower minor faults in the same benchmark log, the page-size evidence reported
`hugepage_observed=no` and `hugepage_adoption=false`.

This is now the preferred compact summary for mapped scratch THP benchmark
logs. Timing remains in the raw Criterion output until repeated-run reporting
is strong enough to parse and compare intervals safely.

## Next Step

Use the report command in the next repeated THP benchmark run. If repeated
evidence ever reports `hugepage_adoption=true`, then update the best-results
note with the same-run timing and page-size proof.
