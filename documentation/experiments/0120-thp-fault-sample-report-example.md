# Experiment 0120: THP Fault Sample Report Example

Date: 2026-07-03

## Postulate

[Postulate 0112: THP Fault Sample Report Example](../postulates/0112-thp-fault-sample-report-example.md)

## Change

Added a command-line example for parsing saved mapped scratch THP fault sample validation output as one coherent report.

The change includes:

- `crates/locus-validate/examples/mapped_scratch_thp_fault_sample_report.rs`;
- README documentation for the report command.

The example reads a file containing the two stable lines emitted by `mapped_scratch_thp_fault_sample_validation_gate`, parses them with `parse_mapped_scratch_thp_fault_sample_report_output`, then prints normalized gate and comparison lines.

## Commands

```text
cargo fmt --all
cargo run -q -p locus-validate --example mapped_scratch_thp_fault_sample_report -- <temp-report-output>
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
```

## Results

The example normalized a ready report:

```text
mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
mapped_scratch_thp_fault_sample_comparison=available reason=ready default_minor_faults_delta=16400 hugepage_minor_faults_delta=8224 no_hugepage_minor_faults_delta=16400 hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false
```

Focused `locus-validate` tests passed:

- host: 43 unit tests passed, plus doc tests.
- Docker: 51 unit tests passed, plus doc tests. The Docker count includes Linux-only placement validation tests.

Workspace validation passed:

- `cargo test --workspace`: 152 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Conclusion

The postulate survived. Users and automation can now validate and normalize saved THP fault sample validation output through a command-line path backed by the typed report parser.

The command still reports only fault sample availability and minor-fault comparison evidence. It does not claim THP adoption or performance superiority.

## Next Questions

- Should report parsing expand next to combine fault sample reports with Criterion timing output?
- Should the report example gain a JSON mode after the stable text schema has more consumers?
