# Experiment 0118: THP Fault Sample Comparison Parser

Date: 2026-07-03

## Postulate

[Postulate 0110: THP Fault Sample Comparison Parser](../postulates/0110-thp-fault-sample-comparison-parser.md)

## Change

Added typed parsing for `mapped_scratch_thp_fault_sample_comparison=` output.

The parser lives in a dedicated `locus-validate` module instead of extending the already large `lib.rs` parser body. Root-level re-exports preserve the public API shape while keeping the comparison output model, display implementation, parser errors, line parser, output parser, and private field accumulator together.

The change includes:

- `crates/locus-validate/src/thp_fault_sample_comparison.rs`;
- `parse_mapped_scratch_thp_fault_sample_comparison_line`;
- `parse_mapped_scratch_thp_fault_sample_comparison_output`;
- `MappedScratchThpFaultSampleComparisonLineParseError`;
- `MappedScratchThpFaultSampleComparisonOutputParseError`;
- root re-exports from `locus_validate`.

The parser accepts:

- available comparison lines with all required delta and boolean fields;
- unavailable comparison lines with no comparison payload;
- multiline command output containing exactly one comparison line.

It rejects missing status, missing reason, unknown status, unknown reason, invalid numbers, invalid booleans, missing available fields, unexpected unavailable fields, duplicate fields, unknown tokens, inconsistent status and reason pairs, missing output lines, duplicate output lines, and malformed output lines.

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

- host: 36 unit tests passed, plus doc tests.
- Docker: 44 unit tests passed, plus doc tests. The Docker count includes Linux-only placement validation tests.

Workspace validation passed:

- `cargo test --workspace`: 145 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Architecture check:

- `crates/locus-validate/src/lib.rs`: 3896 lines after re-exporting the comparison module.
- `crates/locus-validate/src/thp_fault_sample_comparison.rs`: 658 lines holding the comparison-specific output and parser code.

## Conclusion

The postulate survived. `locus-validate` can now parse the THP fault sample comparison line it emits, and the implementation avoids further growing the central validation file with comparison-specific parser internals.

This remains benchmark interpretation infrastructure. Parsed minor-fault comparisons are not THP adoption proof and should still be joined with page-size evidence before any adoption claim.

## Next Questions

- Should other validation gate parsers move into focused modules to reduce `lib.rs` size?
- Should the next report aggregation step consume both the fault sample gate verdict and comparison output together?
