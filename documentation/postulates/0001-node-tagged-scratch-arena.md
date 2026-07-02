# Postulate 0001: Node-Tagged Scratch Arena

Date: 2026-07-02

## Statement

A safe node-tagged scratch arena is a useful first allocator foundation because it validates explicit placement metadata, alignment behavior, reset semantics, and benchmark structure before Locus introduces Linux NUMA memory-policy syscalls.

## Rationale

The research notes identify temporary tensor scratch memory as a short-lived allocation class with reset-oriented lifetime behavior. A bump arena matches that lifetime model and can be tested without unsafe code. The node tag keeps the API aligned with later NUMA binding while the current implementation remains portable and deterministic.

## Experiment

Implement a `ScratchArena` with:

- a home NUMA node tag;
- bounded usable capacity;
- alignment-aware byte allocation;
- reset semantics;
- high-water and allocation counters;
- focused unit tests;
- a Criterion benchmark for repeated allocations followed by reset.

## Expected Result

The arena should pass correctness tests and provide a repeatable benchmark harness. It is not expected to prove NUMA locality yet because no `mbind`, `set_mempolicy`, or page residency validation is implemented in this step.
