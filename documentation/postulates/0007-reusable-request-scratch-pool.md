# Postulate 0007: Reusable Request Scratch Pool

Date: 2026-07-02

## Statement

Reusing closed request scratch arenas by home node and capacity should reduce request-cycle allocation overhead while preserving explicit request ownership and node tagging.

## Rationale

Experiment 0007 showed that request-scoped scratch arenas are safe and explicit, but the benchmark included arena creation in the timed loop. In an inference runtime, request arenas should usually come from per-node reusable pools because request churn is expected under continuous batching.

## Experiment

Add a `RequestScratchPool` that:

- opens request arenas from idle per-node capacity classes when possible;
- creates a new arena only when no matching idle arena exists;
- returns closed arenas to the idle pool;
- resets request-local accounting before reuse;
- reports created and reused arena counts;
- benchmarks the pooled request cycle against the existing fresh-arena and `Vec<u8>` baselines.

## Expected Result

The pooled request cycle should pass correctness tests and provide a clearer benchmark for request churn where arena backing storage is reused.
