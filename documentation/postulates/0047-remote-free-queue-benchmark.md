# Postulate 0047: Remote Free Queue Benchmark

Date: 2026-07-02

## Statement

The `RemoteFreeQueue` primitive should be benchmarked against the persistent-worker handoff baseline.

## Rationale

The primitive added in experiment 0054 provides owner-side batch draining, but it needs a measurement that reflects its intended runtime shape. A persistent producer thread can allocate buffers and enqueue them through a `RemoteFreeSink`, while the benchmark thread acts as the owner and drains batches.

This compares a Locus-owned remote-free shape against the existing persistent-worker channel handoff baseline.

## Experiment

Add a benchmark that:

- starts one persistent producer thread before Criterion iterations;
- allocates 256 zero-filled 4096-byte vectors on the producer thread;
- enqueues each vector into `RemoteFreeQueue`;
- drains batches on the owner thread until all 256 vectors are released.

## Expected Result

The benchmark should compile under all-target checks and produce a measurable baseline. It may be faster or slower than the direct persistent-worker handoff depending on queue batch behavior and owner-side drain overhead.
