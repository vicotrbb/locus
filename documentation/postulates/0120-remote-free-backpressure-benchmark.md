# Postulate 0120: Remote Free Backpressure Benchmark

Date: 2026-07-03

## Statement

The nonblocking remote-free path should have a focused benchmark that measures both completion latency and queue-full backpressure.

## Rationale

`RemoteFreeSink::try_enqueue` gives remote completion paths a way to avoid blocking when the owner has not drained enough work. Unit tests prove the API and counters, but scheduler policy needs timing evidence and congestion counts under a repeatable workload.

The first benchmark should keep the workload generic and small: a persistent remote producer creates 256 KV-sized blocks, attempts nonblocking enqueue, and retries on full-queue backpressure while the owner drains in bounded batches. Comparing small and wider batch limits should show whether larger owner drains reduce retry pressure in the same synthetic shape.

## Experiment

Add a separate Criterion target named `remote_free_backpressure`.

The target should include:

- `remote_free_try_enqueue_backpressure_256x4k_batch8`;
- `remote_free_try_enqueue_backpressure_256x4k_batch64`;
- a persistent remote producer thread;
- owner-side draining through `RemoteFreeQueue::drain_batch`;
- assertions that each iteration drains all 256 blocks;
- benchmark-side observation of `full_count` and `pending_count`.

## Expected Result

The benchmark should compile under all-target checks and produce timing samples. The batch-64 case is expected to show fewer queue-full retries than the batch-8 case, though this synthetic benchmark does not prove an optimal serving policy.
