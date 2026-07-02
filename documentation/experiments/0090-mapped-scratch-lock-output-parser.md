# Experiment 0090: Mapped Scratch Lock Output Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0082-mapped-scratch-lock-output-parser.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_lock
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 28 host-visible unit tests passed, including mapped scratch lock output parser coverage.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 6 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-alloc` passed:

- `locus-alloc`: 29 Linux unit tests passed, including mapped scratch lock output parser coverage.
- Doc tests completed with no failures.

Docker `cargo run -p locus-alloc --example mapped_scratch_lock` output:

```text
mapping_start=0xffffb2e47000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
```

## Conclusion

The postulate survived. `locus-alloc` can now parse mapped scratch page-lock probe status lines and full probe output while ignoring free-form error detail lines.

This makes page-lock readiness consumable by future validation gates for pinned host staging buffers.
