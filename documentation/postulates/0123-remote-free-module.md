# Postulate 0123: Remote Free Module

Date: 2026-07-03

## Statement

The remote-free queue should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

The remote-free queue has become a distinct runtime primitive with blocking enqueue, nonblocking enqueue, backpressure counters, drain accounting, benchmark coverage, and focused tests. Keeping it in `src/lib.rs` makes the allocator root harder to scan and mixes cross-thread release infrastructure with scratch arenas, KV block pools, mapped regions, and probe parsers.

Moving the queue into `remote_free.rs` should reduce root file size, isolate the concurrency imports and error types, and keep the public API source compatible through root re-exports.

## Experiment

Extract the remote-free queue subsystem into `crates/locus-alloc/src/remote_free.rs`.

The module should own:

- `RemoteFreeQueue`;
- `RemoteFreeSink`;
- queue, drain, and enqueue stats;
- blocking and nonblocking enqueue errors;
- queue configuration errors;
- debug, display, and error implementations;
- focused unit tests for draining, invalid configuration, backpressure, and dropped-owner behavior.

## Expected Result

The public `locus_alloc::*` remote-free API should remain source compatible, the root allocator file should shrink, and host plus Docker validation should keep passing.
