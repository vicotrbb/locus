# Postulate 0121: Remote Free Capacity Batch Matrix

Date: 2026-07-03

## Statement

The remote-free backpressure benchmark should separate queue capacity from owner drain batch size.

## Rationale

Experiment 0128 tied queue capacity and drain batch size together. That produced useful timing data, but the one-run `full_count` evidence was hard to interpret because changing from batch8 to batch64 also changed the bounded queue capacity from 8 to 64.

A small two-axis benchmark matrix should keep the existing diagonal cases and add the missing off-diagonal cases. This makes it possible to compare a wider drain at fixed capacity and a wider capacity at fixed drain size before treating queue-full retries as a scheduler signal.

## Experiment

Extend the `remote_free_backpressure` Criterion target with:

- `remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64`;
- `remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8`.

The target should print pre-benchmark sample lines that include both `capacity` and `batch_limit`, then run all four short samples:

- capacity 8, batch 8;
- capacity 8, batch 64;
- capacity 64, batch 8;
- capacity 64, batch 64.

## Expected Result

The benchmark should compile under all-target checks and produce timing plus retry evidence for all four cases. The results should make the `full_count` interpretation narrower and more useful than the diagonal-only benchmark, even if scheduler noise remains visible.
