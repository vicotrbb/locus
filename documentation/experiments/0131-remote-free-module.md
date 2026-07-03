# Experiment 0131: Remote Free Module

Date: 2026-07-03

## Postulate

[Postulate 0123](../postulates/0123-remote-free-module.md) claimed that the remote-free queue can move into a focused `locus-alloc` module while preserving the root public API.

## Change

Extracted the remote-free queue subsystem from `crates/locus-alloc/src/lib.rs` into `crates/locus-alloc/src/remote_free.rs`.

The new module owns:

- `RemoteFreeQueue`;
- `RemoteFreeSink`;
- queue, drain, and enqueue stats;
- blocking and nonblocking enqueue errors;
- queue configuration errors;
- debug, display, and error implementations;
- focused tests for draining, invalid configuration, backpressure, and dropped-owner behavior.

The crate root now re-exports the remote-free API with `pub use remote_free::{...}` so existing callers can continue using `locus_alloc::RemoteFreeQueue` and related types.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
cargo fmt --all -- --check
git diff --check
rg -n "<literal em dash>" documentation crates README.md Cargo.toml Cargo.lock || true
cargo bench -p locus-alloc --bench remote_free_backpressure -- --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-bench-current-target cargo bench -p locus-alloc --bench remote_free_backpressure -- --sample-size 10 --warm-up-time 1 --measurement-time 2
```

## Results

- Host `cargo test -p locus-alloc`: 59 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

Remote-free backpressure benchmark comparison used a clean detached `HEAD`
worktree as the baseline and the same shortened Criterion settings for the
extracted module:

```text
baseline HEAD remote_free_try_enqueue_backpressure_256x4k_batch8: [56.165 us 56.517 us 56.829 us]
current  remote_free_try_enqueue_backpressure_256x4k_batch8: [57.199 us 57.400 us 57.583 us]

baseline HEAD remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64: [55.804 us 56.243 us 56.697 us]
current  remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64: [57.103 us 57.434 us 57.838 us]

baseline HEAD remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8: [54.603 us 54.827 us 54.947 us]
current  remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8: [55.409 us 55.498 us 55.593 us]

baseline HEAD remote_free_try_enqueue_backpressure_256x4k_batch64: [54.333 us 54.531 us 54.756 us]
current  remote_free_try_enqueue_backpressure_256x4k_batch64: [55.393 us 55.475 us 55.571 us]
```

An earlier benchmark against the existing local Criterion history reported
regressions, including one noisy high-outlier run for `capacity8_batch64`.
Repeating from a fresh target directory and comparing to a clean `HEAD`
worktree did not reproduce that large regression. The fresh current run kept
all benchmark samples fully drained with zero pending items.

Line counts after extraction:

```text
5818 crates/locus-alloc/src/lib.rs
 431 crates/locus-alloc/src/remote_free.rs
```

## Conclusion

The postulate survived. The remote-free queue behavior remains covered by focused module tests, benchmark callers keep using the same root API, and the allocator root is smaller and less coupled to concurrency-specific imports.

Next module extraction candidates are the KV block pool, request scratch pool, or probe parser groups. Each should stay behavior-preserving and separately documented.
