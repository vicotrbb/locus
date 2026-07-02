# Postulate 0049: KV Remote Free Batch Size

Date: 2026-07-02

## Statement

KV remote-free queue release should be measured with multiple drain batch sizes before changing the primitive API or scheduler model.

## Rationale

Experiment 0056 established that the first KV remote-free queue path is much slower than same-thread KV block reuse. The queue currently uses a batch limit of 32. Batch size may change the tradeoff between owner-side drain overhead, channel backpressure, and remote completion progress.

Measuring smaller and larger batch limits gives evidence for whether this is a tuning knob worth exposing or optimizing.

## Experiment

Add KV remote-free queue release benchmarks for:

- batch limit 8;
- batch limit 32, preserving the existing benchmark name;
- batch limit 64.

Each benchmark should allocate 256 KV block handles, enqueue them through a persistent remote completion thread, and drain them back to the owning pool.

## Expected Result

The benchmarks should compile under all-target checks. Larger batches are expected to reduce owner drain overhead, but very large batches may not help if channel handoff or allocation dominates.
