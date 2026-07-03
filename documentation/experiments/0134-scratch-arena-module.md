# Experiment 0134: Scratch Arena Module

Date: 2026-07-03

## Postulate

[Postulate 0126](../postulates/0126-scratch-arena-module.md) claimed that the Vec-backed `ScratchArena` can move into a focused `locus-alloc` module while preserving the root public API and real scratch allocation benchmark paths.

## Change

Extracted the base scratch arena subsystem from `crates/locus-alloc/src/lib.rs` into `crates/locus-alloc/src/scratch_arena.rs`.

The new module owns:

- `ScratchArena`;
- `ScratchArenaStats`;
- `ScratchAllocError`;
- private alignment rounding for the Vec-backed arena;
- focused tests for alignment, reset accounting, out-of-memory behavior, and unsupported alignment.

The crate root now re-exports the base scratch API with `pub use scratch_arena::{...}` so existing callers can continue using `locus_alloc::ScratchArena`, `locus_alloc::ScratchArenaStats`, and `locus_alloc::ScratchAllocError`.

Mapped scratch arena code stayed in the root for this experiment.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
CARGO_TARGET_DIR=/tmp/locus-scratch-bench-target cargo bench -p locus-alloc --bench scratch_arena -- scratch_arena_reset_cycle_64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-scratch-bench-target cargo bench -p locus-alloc --bench scratch_arena -- vec_allocation_cycle_64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-scratch-bench-target cargo bench -p locus-alloc --bench scratch_arena -- vec_uninit_capacity_allocation_cycle_64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-scratch-bench-target cargo bench -p locus-alloc --bench scratch_arena -- request_scratch_pool_cycle_16x64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
```

## Results

- Host `cargo test -p locus-alloc`: 59 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.

Focused scratch benchmark results:

```text
scratch_arena_reset_cycle_64x256b: [202.20 ns 202.78 ns 203.23 ns]
vec_allocation_cycle_64x256b: [640.54 ns 643.25 ns 645.62 ns]
vec_uninit_capacity_allocation_cycle_64x256b: [625.89 ns 630.71 ns 634.27 ns]
request_scratch_pool_cycle_16x64x256b: [3.6557 us 3.6590 us 3.6643 us]
```

The first benchmark command also matched the mapped scratch reset benchmark because of Criterion substring filtering:

```text
mapped_scratch_arena_reset_cycle_64x256b: [201.71 ns 203.50 ns 205.48 ns]
```

The extraction did not produce a new best scratch result. The current scratch reset run remains faster than both Vec baselines in this pass, while the best observed scratch arena result remains recorded in `documentation/dev-notes/2026-07-03-best-benchmark-results.md`.

Line counts after extraction:

```text
4724 crates/locus-alloc/src/lib.rs
 259 crates/locus-alloc/src/scratch_arena.rs
 459 crates/locus-alloc/src/request_scratch.rs
 434 crates/locus-alloc/src/kv_block.rs
 431 crates/locus-alloc/src/remote_free.rs
```

## Conclusion

The postulate survived. Base scratch arena behavior remains covered by focused module tests, public callers keep using the same root API, the allocator root is smaller, and real scratch allocation plus request-pool benchmarks still exercise the extracted module successfully.

Next extraction candidates are mapped scratch arena, pinned scratch pool, and parser groups. A future performance step should add repeated clean-baseline scratch benchmark comparison if the arena data layout or alignment arithmetic changes.
