# Experiment 0115: THP Fault Sample Verdict Parser

Date: 2026-07-03

## Postulate

[Postulate 0107: THP Fault Sample Verdict Parser](../postulates/0107-thp-fault-sample-verdict-parser.md)

## Change

Added typed parsing for `mapped_scratch_thp_fault_sample_validation_gate=` verdict output.

The change includes:

- `MappedScratchThpFaultSampleValidationGateVerdict`;
- `MappedScratchThpFaultSampleValidationGateLineParseError`;
- `MappedScratchThpFaultSampleValidationGateOutputParseError`;
- `parse_mapped_scratch_thp_fault_sample_validation_gate_line`;
- `parse_mapped_scratch_thp_fault_sample_validation_gate_output`.

The parser accepts:

- `ready reason=ready`;
- `unavailable reason=fault_counters_unavailable`.

It rejects missing fields, duplicate fields, unknown tokens, extra tokens, and inconsistent status-reason pairs.

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

- host: 32 unit tests passed, plus doc tests.
- Docker: 40 unit tests passed, plus doc tests. The Docker count includes Linux-only placement validation tests.

Workspace validation passed:

- `cargo test --workspace`: 138 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Conclusion

The postulate survived. The THP fault sample validation gate can now emit and parse its stable verdict line, which makes it usable in downstream report aggregation.

The parser validates verdict syntax and consistency only. It does not re-parse the benchmark fault samples behind an already emitted verdict.

## Next Questions

- Should all validation gate verdict parsers be exposed through one trait or helper to reduce repeated token parsing?
- Should saved benchmark reports include both parsed fault samples and parsed gate verdicts in a single summary file?
