# Experiment 0086: Mapped Region Page Lock

Date: 2026-07-02

## Postulate

See `documentation/postulates/0078-mapped-region-page-lock.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 6 host-visible unit tests passed, including page lock and unlock coverage.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-sys` passed:

- `locus-sys`: 16 Linux unit tests passed, including page lock and unlock coverage.
- Doc tests completed with no failures.

## Conclusion

The postulate survived. `MappedRegion` now exposes safe `lock_pages` and `unlock_pages` methods backed by `mlock` and `munlock`.

This is a low-level pinned host memory primitive only. It does not yet implement CUDA host registration, GPU-near pooling, or budgeted staging-buffer checkout.
