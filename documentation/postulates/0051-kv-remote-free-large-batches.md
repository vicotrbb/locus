# Postulate 0051: KV Remote Free Large Batches

Date: 2026-07-02

## Statement

KV remote-free release should be measured at larger batch limits before treating batch 64 as the best observed value.

## Rationale

Experiment 0057 found that batch 64 was faster than batch 32 and batch 8 for a 256-handle KV remote-free workload. That does not show where the improvement stops. Larger batch limits may reduce owner drain overhead further, but they may also increase release latency or stop helping once the full per-iteration handle set fits in the queue.

## Experiment

Add KV remote-free queue release benchmarks for:

- batch limit 128;
- batch limit 256.

Compare them with the earlier batch 64 result.

## Expected Result

The benchmarks should compile under all-target checks. Larger batches may improve throughput in the microbenchmark, but the result should be interpreted alongside release-latency concerns for real schedulers.
