# Experiment 0091: Pinned Scratch Pool

Date: 2026-07-02

## Postulate

[Postulate 0083](../postulates/0083-pinned-scratch-pool.md) claims that Locus should expose a small budgeted pinned scratch pool backed by page-locked mapped scratch arenas.

## Change

Added `PinnedScratchPool` to `locus-alloc`.

The pool:

- owns `MappedScratchArena` instances;
- locks arena mappings lazily on checkout;
- enforces a maximum locked-byte budget;
- returns opaque `PinnedScratchHandle` values;
- exposes checked-out arenas through `get_mut`;
- resets arenas before returning them to the idle pool;
- reports pool accounting through `PinnedScratchPoolStats`.

The implementation remains host-only. It does not register memory with CUDA, bind memory near a GPU, or model async transfer completion.

## Commands

```text
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

`cargo fmt --all` passed.

`cargo test --workspace` passed on the host:

```text
locus-alloc: 32 passed
locus-core: 9 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 0 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-alloc` passed:

```text
locus-alloc: 33 passed
doc tests: passed
```

Docker runs one additional Linux-only allocator test.

## Conclusion

The postulate survived this implementation step. Locus now has a reusable host page-locked scratch pool with explicit locked-byte budgeting and handle-based checkout.

The pool is still not a GPU staging abstraction. Next work should decide whether pinned host buffers need CUDA registration, device-locality policy, or async lifetime tracking before exposing them as GPU-near transfer buffers.
