# Postulate 0045: Allocator Specific Persistent Handoff

Date: 2026-07-02

## Statement

The persistent-worker producer and consumer handoff benchmark should run under mimalloc, jemalloc, and the explicit system allocator.

## Rationale

Experiment 0052 showed that removing thread spawn from the timed path changes the handoff measurement materially for the default allocator benchmark. The allocator-specific handoff results from experiment 0051 still include thread spawn cost, so they are not the best evidence for remote-free research.

Adding persistent-worker handoff cases to the isolated allocator benchmark binaries keeps global allocator identity explicit while better isolating allocation, channel handoff, and remote drop behavior.

## Experiment

Add persistent-worker 256 by 4096-byte handoff benchmarks to:

- `scratch_arena_mimalloc`;
- `scratch_arena_jemalloc`;
- `scratch_arena_system`.

Each benchmark should start a producer and consumer before Criterion iterations, send a run command per iteration, transfer vectors through a bounded channel, and wait for the consumer to report completion.

## Expected Result

The new benchmarks should compile under all-target checks and report lower timings than the matching spawn-per-iteration allocator-specific handoff benchmarks.
