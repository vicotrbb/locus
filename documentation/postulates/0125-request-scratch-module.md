# Postulate 0125: Request Scratch Module

Date: 2026-07-03

## Statement

The request-scoped scratch manager and reusable request scratch pool should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

Request-affine allocation is one of the core Locus runtime directions. `RequestScratch` and `RequestScratchPool` coordinate request homes, per-request arena lifetimes, idle arena reuse, and reuse accounting. Keeping this logic in `src/lib.rs` mixes request scheduling concerns with base arena allocation, mapped regions, probe parsers, KV-cache allocation, and remote-free infrastructure.

Moving the request scratch managers into `request_scratch.rs` should reduce root file size, isolate request-affine lifetime rules and errors, and keep the public API source compatible through root re-exports.

## Experiment

Extract the request scratch subsystem into `crates/locus-alloc/src/request_scratch.rs`.

The module should own:

- `RequestScratch`;
- `RequestScratchPool`;
- `RequestScratchPoolStats`;
- `RequestScratchError`;
- focused unit tests for request open, allocation, reset, close, missing homes, closed requests, idle reuse, and capacity class separation.

The module can depend on the base `ScratchArena` type in the crate root. Any helper required for reuse should remain crate-private and should not become part of the public API.

## Real Workload Gate

The extraction must still pass the existing request-affine benchmark paths:

- `request_scratch_cycle_16x64x256b`;
- `request_scratch_pool_cycle_16x64x256b`;
- `request_vec_allocation_cycle_16x64x256b`;
- `request_remote_free_queue_return_16x64x256b`.

These benchmarks exercise request arena creation, pooled reuse, default Vec allocation, and remote request return through `RemoteFreeQueue`.

## Expected Result

The public `locus_alloc::*` request scratch API should remain source compatible, the root allocator file should shrink, unit tests should keep passing, and real request-affine allocation benchmarks should continue to run successfully.
