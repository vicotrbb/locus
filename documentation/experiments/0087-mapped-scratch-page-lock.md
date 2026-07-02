# Experiment 0087: Mapped Scratch Page Lock

Date: 2026-07-02

## Postulate

See `documentation/postulates/0079-mapped-scratch-page-lock.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 24 host-visible unit tests passed, including mapped scratch page lock and unlock coverage.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 6 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-alloc` passed:

- `locus-alloc`: 25 Linux unit tests passed, including mapped scratch page lock and unlock coverage.
- Doc tests completed with no failures.

## Conclusion

The postulate survived. `MappedScratchArena` now exposes safe `lock_pages` and `unlock_pages` methods that delegate to its owned mapped region and wrap page-locking failures in `MappedScratchAllocError`.

This gives allocator experiments a first pinned-host-memory hook without expanding the unsafe boundary beyond `locus-sys`.
