# Experiment 0119: THP Fault Sample Report Parser

Date: 2026-07-03

## Postulate

[Postulate 0111: THP Fault Sample Report Parser](../postulates/0111-thp-fault-sample-report-parser.md)

## Change

Added a combined typed parser for mapped scratch THP fault sample validation command output.

The change includes:

- `crates/locus-validate/src/thp_fault_sample_report.rs`;
- `MappedScratchThpFaultSampleReport`;
- `MappedScratchThpFaultSampleReportParseError`;
- `parse_mapped_scratch_thp_fault_sample_report_output`;
- root re-exports from `locus_validate`.

The report parser reuses the existing fault sample gate and comparison parsers, then checks that the two parsed lines agree.

Accepted report combinations:

- `ready reason=ready` gate plus `available reason=ready` comparison;
- `unavailable reason=fault_counters_unavailable` gate plus matching unavailable comparison;
- defensive `ready reason=ready` gate plus `unavailable reason=comparison_unavailable` comparison.

Rejected report combinations:

- missing, duplicated, or malformed gate lines;
- missing, duplicated, or malformed comparison lines;
- individually valid lines with contradictory statuses or reasons.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
```

## Results

Focused `locus-validate` tests passed:

- host: 43 unit tests passed, plus doc tests.
- Docker: 51 unit tests passed, plus doc tests. The Docker count includes Linux-only placement validation tests.

Workspace validation passed:

- `cargo test --workspace`: 152 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Conclusion

The postulate survived. Downstream tooling can now parse the two-line THP fault sample validation output as one coherent report instead of coordinating gate and comparison parsers manually.

The report still represents fault sample availability and minor-fault comparison evidence only. It does not prove transparent huge page adoption or performance superiority.

## Next Questions

- Should the report type become the input to a broader saved benchmark report parser that joins timing output and page-size evidence?
- Should the existing validation gate parsers move into focused modules before more report-level aggregation is added?
