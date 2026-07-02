# Postulate 0044: Persistent Worker Handoff Benchmark

Date: 2026-07-02

## Statement

The producer and consumer handoff benchmark should have a persistent-worker variant that removes thread spawn cost from the timed loop.

## Rationale

The current handoff benchmark is useful as a conservative cross-thread baseline, but it starts producer and consumer threads inside each Criterion iteration. In an inference runtime, worker threads are normally long-lived. A persistent-worker benchmark better isolates allocation, bounded-channel handoff, and remote drop costs.

Starting with the default allocator benchmark binary keeps the experiment small before duplicating the shape for mimalloc, jemalloc, and the explicit system allocator.

## Experiment

Add a default allocator benchmark that:

- starts one producer thread and one consumer thread before `bench.iter`;
- sends a run command to the producer on each iteration;
- allocates 256 zero-filled 4096-byte vectors on the producer thread;
- sends vectors through a bounded channel to the consumer;
- drops vectors on the consumer thread;
- reports completion back to the benchmark thread.

## Expected Result

The benchmark should compile under all-target checks and produce a lower handoff time than the spawn-per-iteration baseline because thread creation is no longer in the timed loop.
