# Experiment 0133: Request Scratch Module

Date: 2026-07-03

## Postulate

[Postulate 0125](../postulates/0125-request-scratch-module.md) claimed that the request-scoped scratch manager and reusable request scratch pool can move into a focused `locus-alloc` module while preserving the root public API and real request-affine benchmark paths.

## Change

Extracted the request scratch subsystem from `crates/locus-alloc/src/lib.rs` into `crates/locus-alloc/src/request_scratch.rs`.

The new module owns:

- `RequestScratch`;
- `RequestScratchPool`;
- `RequestScratchPoolStats`;
- `RequestScratchError`;
- focused tests for request open, allocation, reset, close, missing homes, closed requests, idle reuse, and capacity class separation.

The crate root now re-exports the request scratch API with `pub use request_scratch::{...}` so existing callers can continue using `locus_alloc::RequestScratch`, `locus_alloc::RequestScratchPool`, and related types.

The only base arena visibility change was making `ScratchArena::prepare_for_reuse` crate-private so the extracted pool can reset internal reuse accounting without adding a public API.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
CARGO_TARGET_DIR=/tmp/locus-request-bench-target cargo bench -p locus-alloc --bench scratch_arena -- request_scratch_cycle_16x64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-request-bench-target cargo bench -p locus-alloc --bench scratch_arena -- request_vec_allocation_cycle_16x64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-request-bench-target cargo bench -p locus-alloc --bench scratch_arena -- request_scratch_pool_cycle_16x64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-request-bench-target cargo bench -p locus-alloc --bench scratch_arena -- request_remote_free_queue_return_16x64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
```

## Results

- Host `cargo test -p locus-alloc`: 59 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.

Focused request benchmark results:

```text
request_scratch_cycle_16x64x256b: [6.8342 us 6.8457 us 6.8570 us]
request_vec_allocation_cycle_16x64x256b: [12.175 us 12.208 us 12.262 us]
request_scratch_pool_cycle_16x64x256b: [3.2059 us 3.2109 us 3.2154 us]
request_remote_free_queue_return_16x64x256b: [6.7961 us 6.8198 us 6.8401 us]
```

The extraction did not produce a new best request scratch result. The current pooled request scratch run remains faster than the Vec baseline and non-pooled request scratch path in this pass, while the best observed request-affine reuse result remains recorded in `documentation/dev-notes/2026-07-03-best-benchmark-results.md`.

Line counts after extraction:

```text
4960 crates/locus-alloc/src/lib.rs
 459 crates/locus-alloc/src/request_scratch.rs
 434 crates/locus-alloc/src/kv_block.rs
 431 crates/locus-alloc/src/remote_free.rs
```

## Conclusion

The postulate survived. Request-affine allocation behavior remains covered by focused module tests, public callers keep using the same root API, the allocator root is smaller, and real request allocation plus remote-return benchmarks still exercise the extracted module successfully.

Next extraction candidates are the base scratch arena, mapped scratch arena, pinned scratch pool, and parser groups. A future performance step should add repeated clean-baseline request benchmark comparison if request arena data structures change.
