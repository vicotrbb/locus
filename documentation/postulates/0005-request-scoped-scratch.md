# Postulate 0005: Request-Scoped Scratch Arenas

Date: 2026-07-02

## Statement

A request-scoped scratch manager can make request-affine allocation explicit by opening one node-tagged arena per active request and closing it at request completion.

## Rationale

The current scratch arena validates reset-oriented temporary allocation, but it is not tied to request identity. LLM serving needs request-affine allocation so scheduler decisions, placement metadata, and allocation accounting remain connected.

## Experiment

Add a `RequestScratch` manager that:

- opens a `ScratchArena` from a `RequestHome`;
- rejects requests without a home node;
- allocates, resets, and closes by `RequestId`;
- reports final arena stats at close;
- adds a benchmark for a multi-request scratch cycle.

## Expected Result

The manager should pass focused tests and preserve the safe Rust boundary. The benchmark should provide an early request-affinity workload shape while still being explicit that the arena backing is not NUMA-bound yet.
