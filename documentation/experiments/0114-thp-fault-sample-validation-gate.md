# Experiment 0114: THP Fault Sample Validation Gate

Date: 2026-07-03

## Postulate

[Postulate 0106: THP Fault Sample Validation Gate](../postulates/0106-thp-fault-sample-validation-gate.md)

## Change

Added a validation gate for mapped scratch THP benchmark fault sample logs.

The change includes:

- `MappedScratchThpFaultSampleValidationGateStatus`;
- `MappedScratchThpFaultSampleValidationGateReason`;
- `MappedScratchThpFaultSampleValidationGate`;
- `evaluate_mapped_scratch_thp_fault_sample_validation_output`;
- `mapped_scratch_thp_fault_sample_validation_gate` example.

The gate reports:

- `ready` when the `default`, `hugepage`, and `no_hugepage` fault samples are present and available;
- `unavailable` when the sample set is complete but one or more samples reported unavailable process fault counters.

Malformed, duplicate, or incomplete sample output remains a parser error.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo run -p locus-validate --example mapped_scratch_thp_fault_sample_validation_gate -- <temp-benchmark-output>
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
```

## Results

Focused `locus-validate` tests passed:

- host: 28 unit tests passed, plus doc tests.
- Docker: 36 unit tests passed, plus doc tests. The Docker count includes Linux-only placement validation tests.

The file-based example printed:

```text
mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
```

Workspace validation passed:

- `cargo test --workspace`: 134 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Conclusion

The postulate survived. Saved THP benchmark logs can now be checked for complete and available fault sample evidence with one stable verdict line.

This gate validates fault sample availability only. It does not validate Criterion timing output and does not prove transparent huge page adoption.

## Next Questions

- Should this gate parse its own verdict line for downstream report aggregation?
- Should benchmark runs write fault samples to a separate sidecar file to avoid mixing diagnostics with Criterion output?
