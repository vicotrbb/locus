# Experiment 0126: Linux Placement Gate Module

Date: 2026-07-03

## Postulate

[Postulate 0118](../postulates/0118-linux-placement-gate-module.md) claimed that the Linux placement validation gate can move into a Linux submodule while preserving the existing `locus_validate::linux::*` API.

## Change

Extracted the Linux placement validation gate from `crates/locus-validate/src/lib.rs` into `crates/locus-validate/src/linux/placement_validation_gate.rs`.

Added `crates/locus-validate/src/linux/mod.rs` as the Linux API surface and re-exported the placement validation gate types and functions from it.

The extracted placement gate module owns:

- placement validation input, gate, verdict, status, and reason types;
- line, output, and probe parse errors;
- display implementations;
- combined evaluation from Linux memory policy, placement readiness, and placement proof outputs;
- stable gate line and multiline output parsing;
- focused Linux-only tests.

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
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate`: 51 unit tests passed, including `linux::placement_validation_gate` tests, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

Line counts after extraction:

```text
 57 crates/locus-validate/src/lib.rs
 11 crates/locus-validate/src/linux/mod.rs
791 crates/locus-validate/src/linux/placement_validation_gate.rs
715 crates/locus-validate/src/pinned_scratch_near_gpu_gate.rs
748 crates/locus-validate/src/pinned_scratch_gate.rs
```

## Conclusion

The postulate survived. The root validation file is now a compact crate-level API surface, the Linux placement validation API remains available through `locus_validate::linux::*`, and Docker validation proves the platform-specific module compiles and passes its Linux tests.

Next, the remaining work can move back from validation-file organization toward allocator behavior and benchmark-facing experiments.
