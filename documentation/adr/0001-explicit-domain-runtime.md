# ADR 0001: Use Explicit Domain Allocators

Date: 2026-07-02

## Status

Accepted

## Context

The existing research argues that Locus should start as a memory locality runtime for inference workloads, not as a global allocator replacement. The workload has distinct memory classes: request metadata, private KV cache, shared prefix KV cache, read-mostly mapped weights, scratch tensors, and pinned host staging buffers.

Global allocator replacement would make attribution, placement policy, request affinity, and benchmark isolation harder at this stage.

## Decision

Locus will begin with explicit domain crates:

- `locus-core` for topology, policy, memory class, and placement models.
- `locus-topology` for Linux sysfs topology discovery.
- `locus-alloc` for focused allocator experiments.

The first allocator experiment is a safe node-tagged scratch arena backed by `Vec<u8>`. Linux NUMA binding and raw memory mapping are deferred until the safe API, tests, and benchmark harness are stable.

## Consequences

- The public API keeps memory class, placement, and lifetime visible.
- Unsafe code is not needed for the initial foundation.
- Future Linux syscalls can be isolated behind a narrow implementation layer.
- Benchmarks can compare each domain allocator against simple Rust baselines before any global allocator experiment.
