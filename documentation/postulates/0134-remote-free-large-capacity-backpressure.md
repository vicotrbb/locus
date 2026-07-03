# Postulate 0134: Remote-Free Large Capacity Backpressure

Date: 2026-07-03

## Statement

Increasing `RemoteFreeQueue` capacity beyond 64 entries should keep full-queue retry counts at zero or near zero for the 256 by 4 KiB backpressure workload, but it may not improve timing once capacity is already large enough to avoid producer stalls.

## Rationale

Experiment 0130 showed that capacity 64 with batch 64 produced zero full-queue retries across repeated pre-benchmark samples and the best timing interval in that run. The workload returns 256 blocks per iteration, so larger capacities should test whether extra buffering reduces scheduling sensitivity further or only adds memory footprint without useful timing benefit.

This matters for runtime policy. A queue capacity that is too small can introduce producer backpressure. A queue capacity that is larger than needed can consume memory and hide release latency without improving throughput.

## Experiment

Extend `crates/locus-alloc/benches/remote_free_backpressure.rs` with:

- `remote_free_try_enqueue_backpressure_256x4k_capacity128_batch64`;
- `remote_free_try_enqueue_backpressure_256x4k_capacity256_batch64`.

Each case should print the same one-run sample and eight-run summary lines as the existing capacity and batch cases.

## Expected Result

The larger-capacity cases should compile under all-target checks and produce parseable benchmark evidence. If either larger capacity beats the existing best nonblocking remote-free backpressure timing, record it in the best-results note. If not, keep the current best and use the result to bound useful capacity growth.
