# Experiment 0124: Pinned Scratch Gate Module

Date: 2026-07-03

## Postulate

[Postulate 0116](../postulates/0116-pinned-scratch-gate-module.md) claimed that the base pinned scratch validation gate can move into a dedicated `locus-validate` module while preserving root re-exports and stable behavior.

## Change

Extracted the host page-locked pinned scratch validation gate from `crates/locus-validate/src/lib.rs` into `crates/locus-validate/src/pinned_scratch_gate.rs`.

The new module owns:

- gate status, reason, gate, and verdict types;
- line, output, and evaluation parse errors;
- display implementations;
- pinned scratch pool probe evaluation;
- line and multiline gate parsers;
- focused tests for ready, not-ready, parser-error, duplicate, malformed, and inconsistent cases.

The near-GPU pinned scratch gate remains in the root file for a separate extraction because it has distinct topology and availability semantics.

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
1570 crates/locus-validate/src/lib.rs
 748 crates/locus-validate/src/pinned_scratch_gate.rs
 729 crates/locus-validate/src/mapped_scratch_thp_gate.rs
 724 crates/locus-validate/src/thp_fault_sample_gate.rs
```

## Conclusion

The postulate survived. The base pinned scratch gate behavior remains covered by focused module tests, the root validation file is smaller, and the public crate API remains source compatible through root re-exports.

Next, the near-GPU pinned scratch validation gate can be extracted without mixing base pinned scratch behavior into that change.
