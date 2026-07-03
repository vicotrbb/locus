# Experiment 0113: THP Benchmark Fault Sample Parser

Date: 2026-07-03

## Postulate

[Postulate 0105: THP Benchmark Fault Sample Parser](../postulates/0105-thp-benchmark-fault-sample-parser.md)

## Change

Added typed parsing for mapped scratch THP benchmark fault sample output in `locus-alloc`.

The change includes:

- `MappedScratchThpFaultSampleMode`;
- `MappedScratchThpFaultSampleStatus`;
- `MappedScratchThpFaultSampleLine`;
- `MappedScratchThpFaultSamples`;
- `parse_mapped_scratch_thp_fault_sample_line`;
- `parse_mapped_scratch_thp_fault_samples_output`.

The multiline parser ignores Criterion timing lines and consumes only `fault_sample=` lines. It requires exactly one `default`, `hugepage`, and `no_hugepage` sample.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

Focused `locus-alloc` tests passed:

- host: 55 unit tests passed, plus doc tests.
- Docker: 57 unit tests passed, plus doc tests.

Workspace tests passed:

- `cargo test --workspace`: 131 unit tests passed across workspace crates, plus doc tests.

Clippy initially rejected the first parser implementation because `parse_mapped_scratch_thp_fault_sample_line` exceeded the line-count lint. The parser was refactored into a small accumulator with token parsing and finish phases. The final clippy run passed with `-D warnings`.

## Conclusion

The postulate survived. THP benchmark fault sample output now has typed line and multiline parsers that accept the current benchmark output and reject missing, duplicate, unknown, malformed, and internally inconsistent samples.

The parser covers the diagnostic `fault_sample=` lines only. It does not parse Criterion timing summaries.

## Next Questions

- Should a validation example consume a saved benchmark log and report one verdict for sample completeness?
- Should future benchmark diagnostics include a run identifier so fault samples can be matched to Criterion reports across repeated runs?
