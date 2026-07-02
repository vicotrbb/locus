# Postulate 0048: KV Remote Free Queue Benchmark

Date: 2026-07-02

## Statement

The `RemoteFreeQueue` primitive should be measured with a KV block pool handle release workload.

## Rationale

The queue benchmark in experiment 0055 releases generic vectors. LLM inference runtimes need domain allocator evidence, especially for KV-cache block lifetimes where completion or scheduler paths may release handles from a different thread than the owning pool.

A benchmark that allocates KV block handles from `KvBlockPool`, sends them through a remote completion thread, and drains the handles back to the owning pool is a closer test of the intended remote-free path.

## Experiment

Add a benchmark that:

- owns a `KvBlockPool` on the benchmark thread;
- allocates 256 KV block handles per iteration;
- sends the handles to a persistent remote completion thread;
- enqueues each handle into `RemoteFreeQueue`;
- drains the queue on the owner thread and calls `KvBlockPool::free`.

## Expected Result

The benchmark should compile under all-target checks and produce a measurable domain remote-free baseline. It is expected to be slower than direct same-thread `kv_block_pool_cycle_256x4k` because it adds a remote completion thread, channel handoff, and queue draining.
