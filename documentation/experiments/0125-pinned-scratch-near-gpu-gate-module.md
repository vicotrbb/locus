# Experiment 0125: Pinned Scratch Near-GPU Gate Module

Date: 2026-07-03

## Postulate

[Postulate 0117](../postulates/0117-pinned-scratch-near-gpu-gate-module.md) claimed that the near-GPU pinned scratch validation gate can move into a dedicated `locus-validate` module while preserving root re-exports and stable behavior.

## Change

Extracted the near-GPU pinned scratch validation gate from `crates/locus-validate/src/lib.rs` into `crates/locus-validate/src/pinned_scratch_near_gpu_gate.rs`.

The new module owns:

- gate status, reason, gate, and verdict types;
- line and evaluation parse errors;
- display implementations;
- near-GPU pinned scratch probe evaluation;
- stable gate line parsing;
- focused tests for ready, unavailable, not-ready, accounting-failure, parser-error, and inconsistent cases.

The crate root now declares and re-exports the module while keeping Linux placement validation separate.

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
862 crates/locus-validate/src/lib.rs
715 crates/locus-validate/src/pinned_scratch_near_gpu_gate.rs
748 crates/locus-validate/src/pinned_scratch_gate.rs
729 crates/locus-validate/src/mapped_scratch_thp_gate.rs
724 crates/locus-validate/src/thp_fault_sample_gate.rs
```

## Conclusion

The postulate survived. The near-GPU pinned scratch gate behavior remains covered by focused module tests, the public crate API remains source compatible through root re-exports, and the root validation file now primarily coordinates modules before the Linux placement gate.

Next, the Linux placement validation gate can be considered for a focused module extraction so the crate root becomes a compact public API surface.
