# Experiment 0122: THP Fault Sample Gate Module

Date: 2026-07-03

## Postulate

[Postulate 0114](../postulates/0114-thp-fault-sample-gate-module.md) claimed that the mapped scratch THP fault sample validation gate can move into a dedicated `locus-validate` module while preserving the public API and parser behavior.

## Change

Extracted the mapped scratch THP fault sample validation gate from `crates/locus-validate/src/lib.rs` into `crates/locus-validate/src/thp_fault_sample_gate.rs`.

The new module owns:

- gate status, reason, gate, and verdict types;
- line, output, and evaluation parse errors;
- display implementations;
- benchmark output evaluation;
- line and multiline gate parsers;
- focused tests for ready, unavailable, parser-error, duplicate, malformed, and inconsistent cases.

The crate root now re-exports the same public names from the focused module.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker compose run --rm locus-dev cargo test -p locus-validate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
cargo fmt --all -- --check
git diff --check
rg -n "<literal em dash>" documentation crates README.md Cargo.toml Cargo.lock || true
```

## Results

- Host `cargo test -p locus-validate`: 43 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker compose command failed because the repository has no compose file at the root.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate`: 51 unit tests passed, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

Line counts after extraction:

```text
3008 crates/locus-validate/src/lib.rs
 724 crates/locus-validate/src/thp_fault_sample_gate.rs
 858 crates/locus-validate/src/thp_fault_sample_comparison.rs
 295 crates/locus-validate/src/thp_fault_sample_report.rs
```

## Conclusion

The postulate survived. The root validation file is smaller, the THP fault sample gate behavior remains covered by focused tests, and the public crate API remains available through root re-exports.

Next, the broad pinned scratch and mapped THP validation gate tests should be evaluated for similar module extraction before adding more parser or report behavior.
