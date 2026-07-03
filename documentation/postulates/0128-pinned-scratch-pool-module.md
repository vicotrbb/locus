# Postulate 0128: Pinned Scratch Pool Module

Date: 2026-07-03

## Statement

The page-locked `PinnedScratchPool` runtime should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

The pinned scratch pool is allocator runtime code. It manages lazily created page-locked mapped scratch arenas, locked-byte budgets, checkout handles, release reuse, and near-GPU NUMA policy resolution. Keeping this logic in `src/lib.rs` mixes a stateful allocation pool with stable probe output schemas, probe parsers, mapped THP parser code, and validation-facing text parsing.

Moving the pool into `pinned_scratch.rs` should reduce root-file size, keep page-locked allocation state close to its tests, and preserve the public API through root re-exports.

## Experiment

Extract the pinned scratch pool subsystem into `crates/locus-alloc/src/pinned_scratch.rs`.

The module should own:

- `PinnedScratchPool`;
- `PinnedScratchHandle`;
- `PinnedScratchPoolStats`;
- `PinnedScratchPoolError`;
- focused tests for checkout reuse, locked-byte budget enforcement, invalid configuration, invalid handles, near-GPU topology resolution, and missing GPU locality.

The stable probe output schemas and parsers should remain in the root for this experiment. They can move later as a parser-focused extraction.

## Real Workload Gate

The extraction must still pass existing pinned scratch validation paths:

- `pinned_scratch_pool_reuse_cycle_64x256b`;
- `pinned_scratch_pool` example in Docker;
- `live_pinned_scratch_validation_gate` example in Docker;
- `pinned_scratch_near_gpu` parser and validation tests.

These exercise page-locked mapped allocation, handle reuse, budget enforcement, stable pool output, and validation-gate compatibility.

## Expected Result

The public `locus_alloc::*` pinned scratch pool API should remain source compatible, the root allocator file should shrink, unit tests should keep passing, Docker examples should still produce stable pinned scratch output, and the pinned scratch reuse benchmark should continue to run successfully.
