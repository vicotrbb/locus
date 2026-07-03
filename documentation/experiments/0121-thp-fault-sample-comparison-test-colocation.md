# Experiment 0121: THP Fault Sample Comparison Test Colocation

Date: 2026-07-03

## Postulate

[Postulate 0113: THP Fault Sample Comparison Test Colocation](../postulates/0113-thp-fault-sample-comparison-test-colocation.md)

## Change

Moved THP fault sample comparison parser tests from the broad `locus-validate` root test module into `crates/locus-validate/src/thp_fault_sample_comparison.rs`.

The moved coverage includes:

- valid available and unavailable comparison lines;
- invalid status, reason, number, bool, missing-field, duplicate-field, unknown-field, and inconsistent status-reason cases;
- valid multiline comparison extraction;
- missing, duplicate, and malformed multiline comparison output.

This keeps comparison parser behavior and its schema edge cases beside the parser implementation while preserving root-level public re-exports.

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

Architecture check:

- `crates/locus-validate/src/lib.rs`: 3708 lines after moving comparison parser tests out.
- `crates/locus-validate/src/thp_fault_sample_comparison.rs`: 858 lines after taking ownership of its parser tests.

## Conclusion

The postulate survived. The comparison parser tests now live with the comparison parser, reducing root test-module coupling without changing public behavior or test coverage.

## Next Questions

- Should the fault sample validation gate parser receive the same module extraction treatment?
- Should parser tests move first for other validation domains before moving their implementation code?
