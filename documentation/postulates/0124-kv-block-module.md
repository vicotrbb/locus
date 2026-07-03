# Postulate 0124: KV Block Module

Date: 2026-07-03

## Statement

The KV block pool and logical KV block table should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

The KV block pool is an AI inference specific allocator primitive with fixed-size block ownership, stale-handle protection, logical token table growth, pool exhaustion rollback, and benchmark coverage against Vec-backed allocation paths. Keeping it in `src/lib.rs` mixes KV-cache allocation behavior with scratch arenas, mapped regions, probe parsers, and remote-free infrastructure.

Moving the KV block pool and table into `kv_block.rs` should reduce root file size, isolate KV-cache allocation types and errors, and keep the public API source compatible through root re-exports.

## Experiment

Extract the KV block subsystem into `crates/locus-alloc/src/kv_block.rs`.

The module should own:

- `KvBlockPool`;
- `KvBlockHandle`;
- `KvBlockTable`;
- `KvSequenceId`;
- pool and table stats;
- pool and table errors;
- token-to-block rounding;
- focused unit tests for reuse, stale handles, invalid configuration, table growth, rollback, and release.

## Real Workload Gate

The extraction must still pass the existing KV-cache benchmark paths that allocate 256 4 KiB blocks, compare against Vec allocation, grow logical block tables, and release block handles through the remote-free queue. A clean `HEAD` comparison should be used when Criterion reports a surprising change.

## Expected Result

The public `locus_alloc::*` KV block API should remain source compatible, the root allocator file should shrink, unit tests should keep passing, and real KV allocation benchmarks should continue to run successfully.
