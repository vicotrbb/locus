# Experiment 0136: Pinned Scratch Pool Module

Date: 2026-07-03

Postulate: [0128 Pinned Scratch Pool Module](../postulates/0128-pinned-scratch-pool-module.md)

## Goal

Extract the page-locked pinned scratch pool runtime from the broad `locus-alloc` root file into a focused module while preserving the public `locus_alloc::*` API, stable probe output, validation gates, and measured reuse behavior.

## Change

Added `crates/locus-alloc/src/pinned_scratch.rs` and moved the pinned scratch pool subsystem into it:

- `PinnedScratchPool`;
- `PinnedScratchHandle`;
- `PinnedScratchPoolStats`;
- `PinnedScratchPoolError`;
- focused tests for checkout reuse, locked-byte budget enforcement, invalid configuration, invalid handles, near-GPU topology resolution, missing GPU topology, and GPU topology without a NUMA node.

The crate root now declares `mod pinned_scratch;` and re-exports the public pinned scratch pool API. Stable probe output schemas and parser logic remain in `src/lib.rs` for a later parser-focused extraction.

## Size Result

| File | Lines |
| --- | ---: |
| `crates/locus-alloc/src/lib.rs` before extraction | 4287 |
| `crates/locus-alloc/src/lib.rs` after extraction | 3741 |
| `crates/locus-alloc/src/pinned_scratch.rs` after extraction | 563 |

The root file is still large, but the extraction removed 546 lines from it without changing the public pinned scratch pool API path.

## Validation Commands

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-alloc --example pinned_scratch_pool
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-validate --example live_pinned_scratch_validation_gate
rm -rf /tmp/locus-pinned-bench-target
CARGO_TARGET_DIR=/tmp/locus-pinned-bench-target cargo bench -p locus-alloc --bench scratch_arena -- pinned_scratch_pool_reuse_cycle_64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- pinned_scratch_pool_reuse_cycle_64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
```

## Test Results

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `cargo test -p locus-alloc`: passed, 61 tests plus doc tests.

Docker `pinned_scratch_pool` result:

```text
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_reuse_checkout=ok handle=1
pool_stats phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_reuse_release=ok handle=1
pool_stats phase=after_reuse_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
```

Docker `live_pinned_scratch_validation_gate` result ended with:

```text
pinned_scratch_validation_gate=ready reason=ready
```

## Benchmark Results

Host pinned scratch reuse benchmark:

| Benchmark | Result |
| --- | ---: |
| `pinned_scratch_pool_reuse_cycle_64x256b` | 197.84 ns to 199.14 ns |

Docker Linux pinned scratch reuse benchmark:

| Benchmark | Result |
| --- | ---: |
| `pinned_scratch_pool_reuse_cycle_64x256b` | 220.79 ns to 223.09 ns |

The host run is the fastest pinned scratch reuse range observed so far. The Docker run matches the earlier Linux-shaped result from Experiment 0094 and confirms the module extraction did not break the page-locked reuse path.

## Conclusion

The postulate survived. `PinnedScratchPool` can be owned by a focused module with source-compatible root re-exports, preserved unit coverage, preserved Docker page-lock validation, preserved validation-gate behavior, and preserved steady-state reuse benchmark behavior.

The next root extraction should target stable probe schemas and parsers as a separate parser module, since `src/lib.rs` is now primarily parser and output-schema code.
