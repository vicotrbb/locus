# Postulate 0042: Producer Consumer Handoff Benchmark

Date: 2026-07-02

## Statement

The benchmark suite should include a cross-thread producer and consumer allocation handoff before Locus designs remote-free batching.

## Rationale

Inference runtimes often allocate buffers on one worker path and release them on another scheduler, networking, or completion path. Single-thread allocation microbenchmarks do not expose allocator behavior under this producer and consumer shape.

A simple channel handoff benchmark gives Locus a first repeatable workload for cross-thread allocation and drop behavior. It is intentionally a baseline workload, not a final remote-free implementation.

## Experiment

Add a benchmark that:

- spawns a producer thread and consumer thread per iteration;
- allocates 256 zero-filled 4096-byte vectors on the producer thread;
- sends each vector through a bounded channel;
- drops the vectors on the consumer thread.

## Expected Result

The benchmark should compile under all-target checks and produce a repeatable baseline. It is expected to be much slower than single-thread KV block pool reuse because it includes thread spawn, channel handoff, allocation, and cross-thread drop costs.
