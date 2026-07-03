# Experiment 0123: Mapped Scratch THP Gate Module

Date: 2026-07-03

## Postulate

[Postulate 0115](../postulates/0115-mapped-scratch-thp-gate-module.md) claimed that the mapped scratch THP validation gate can move into a dedicated `locus-validate` module while preserving root re-exports and stable behavior.

## Change

Extracted the mapped scratch THP validation gate from `crates/locus-validate/src/lib.rs` into `crates/locus-validate/src/mapped_scratch_thp_gate.rs`.

The new module owns:

- gate status, reason, gate, and verdict types;
- line, output, and evaluation parse errors;
- display implementations;
- probe output evaluation;
- line and multiline gate parsers;
- focused tests for ready, unavailable, not-ready, parser-error, duplicate, malformed, and inconsistent cases.

The crate root now re-exports the same public names from the focused module.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
cargo fmt --all -- --check
git diff --check
rg -n "<literal em dash>" documentation crates README.md Cargo.toml Cargo.lock || true
```

## Results

- Host `cargo test -p locus-validate`: 43 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate`: 51 unit tests passed, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

Line counts after extraction:

```text
2299 crates/locus-validate/src/lib.rs
 729 crates/locus-validate/src/mapped_scratch_thp_gate.rs
 724 crates/locus-validate/src/thp_fault_sample_gate.rs
 858 crates/locus-validate/src/thp_fault_sample_comparison.rs
 295 crates/locus-validate/src/thp_fault_sample_report.rs
```

## Conclusion

The postulate survived. The mapped scratch THP gate behavior remains covered by focused module tests, the root validation file is smaller, and the public crate API remains source compatible through root re-exports.

Next, the remaining pinned scratch validation gate families should be considered for focused module extraction before more validation behavior is added to `lib.rs`.
