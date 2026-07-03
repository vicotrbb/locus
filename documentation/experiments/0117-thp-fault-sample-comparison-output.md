# Experiment 0117: THP Fault Sample Comparison Output

Date: 2026-07-03

## Postulate

[Postulate 0109: THP Fault Sample Comparison Output](../postulates/0109-thp-fault-sample-comparison-output.md)

## Change

Added displayable mapped scratch THP benchmark fault sample comparison output to `locus-validate`.

The change includes:

- `MappedScratchThpFaultSampleComparisonStatus`;
- `MappedScratchThpFaultSampleComparisonReason`;
- `MappedScratchThpFaultSampleComparisonOutput`;
- `MappedScratchThpFaultSampleValidationGate::comparison_output`;
- a second output line from the `mapped_scratch_thp_fault_sample_validation_gate` example;
- a README note that the command now prints both the sample availability gate and the comparison line.

The available comparison line uses this schema:

```text
mapped_scratch_thp_fault_sample_comparison=available reason=ready default_minor_faults_delta=<i128> hugepage_minor_faults_delta=<i128> no_hugepage_minor_faults_delta=<i128> hugepage_vs_default_minor_faults_delta=<i128> hugepage_vs_no_hugepage_minor_faults_delta=<i128> major_faults_observed=<bool>
```

The unavailable comparison line uses this schema:

```text
mapped_scratch_thp_fault_sample_comparison=unavailable reason=<fault_counters_unavailable|comparison_unavailable>
```

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -q -p locus-validate --example mapped_scratch_thp_fault_sample_validation_gate -- <temp-benchmark-output>
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
```

## Results

Focused `locus-validate` tests passed:

- host: 32 unit tests passed, plus doc tests.
- Docker: 40 unit tests passed, plus doc tests. The Docker count includes Linux-only placement validation tests.

Workspace validation passed:

- `cargo test --workspace`: 141 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Example output from a temporary benchmark sample:

```text
mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
mapped_scratch_thp_fault_sample_comparison=available reason=ready default_minor_faults_delta=16400 hugepage_minor_faults_delta=8224 no_hugepage_minor_faults_delta=16400 hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false
```

## Conclusion

The postulate survived. Saved mapped scratch THP benchmark logs can now be passed through one validation command that reports both fault sample availability and a stable process minor-fault comparison.

The comparison remains benchmark interpretation evidence only. It does not prove THP adoption, timing superiority, or GPU-near placement.

## Next Questions

- Should `locus-validate` add a parser for `mapped_scratch_thp_fault_sample_comparison=` lines before report aggregation consumes them?
- Should comparison output include child minor-fault deltas, or should it stay focused on process counters until benchmarks show a need?
