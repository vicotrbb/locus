# Experiment 0132: KV Block Module

Date: 2026-07-03

## Postulate

[Postulate 0124](../postulates/0124-kv-block-module.md) claimed that the KV block pool and logical KV block table can move into a focused `locus-alloc` module while preserving the root public API and the real KV allocation benchmark paths.

## Change

Extracted the KV block subsystem from `crates/locus-alloc/src/lib.rs` into `crates/locus-alloc/src/kv_block.rs`.

The new module owns:

- `KvBlockPool`;
- `KvBlockHandle`;
- `KvBlockTable`;
- `KvSequenceId`;
- pool and table stats;
- pool and table errors;
- token-to-block rounding;
- focused tests for reuse, stale handles, invalid configuration, table growth, rollback, and release.

The crate root now re-exports the KV block API with `pub use kv_block::{...}` so existing callers can continue using `locus_alloc::KvBlockPool`, `locus_alloc::KvBlockTable`, and related types.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
CARGO_TARGET_DIR=/tmp/locus-kv-bench-target cargo bench -p locus-alloc --bench scratch_arena -- kv_block_pool_cycle_256x4k --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-kv-bench-target cargo bench -p locus-alloc --bench scratch_arena -- kv_vec_allocation_cycle_256x4k --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-kv-bench-target cargo bench -p locus-alloc --bench scratch_arena -- kv_vec_uninit_capacity_allocation_cycle_256x4k --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-kv-bench-target cargo bench -p locus-alloc --bench scratch_arena -- kv_block_table_append_release_128x16tokens --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-kv-bench-target cargo bench -p locus-alloc --bench scratch_arena -- kv_remote_free_queue_release_256x4k --sample-size 10 --warm-up-time 1 --measurement-time 2
```

## Results

- Host `cargo test -p locus-alloc`: 59 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.

Focused KV benchmark results:

```text
kv_block_pool_cycle_256x4k: [1.1755 us 1.1772 us 1.1785 us]
kv_vec_allocation_cycle_256x4k: [17.076 us 17.138 us 17.219 us]
kv_vec_uninit_capacity_allocation_cycle_256x4k: [5.7554 us 5.7691 us 5.7899 us]
kv_block_table_append_release_128x16tokens: [1.9195 us 1.9225 us 1.9259 us]
kv_remote_free_queue_release_256x4k: [20.628 us 20.698 us 20.794 us]
```

The extraction did not produce a new best KV block result. The current `kv_block_pool_cycle_256x4k` run is still faster than both Vec baselines in this pass, while the best observed KV reuse result remains recorded in `documentation/dev-notes/2026-07-03-best-benchmark-results.md`.

Line counts after extraction:

```text
5399 crates/locus-alloc/src/lib.rs
 434 crates/locus-alloc/src/kv_block.rs
 431 crates/locus-alloc/src/remote_free.rs
```

## Conclusion

The postulate survived. The KV block pool and logical table behavior remain covered by focused module tests, public callers keep using the same root API, the allocator root is smaller, and real KV allocation benchmarks still exercise the extracted module successfully.

Next extraction candidates are request scratch pool, mapped scratch arena, and parser groups. The next performance step should add a repeated clean-baseline KV benchmark comparison if any code changes alter the KV allocation data structures.
