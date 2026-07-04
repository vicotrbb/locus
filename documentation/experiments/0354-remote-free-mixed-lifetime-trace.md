# Experiment 0354: Mixed-Lifetime KV Trace, Sharded Chunk vs Shared Queue

Date: 2026-07-04

## Postulate

[Postulate 0354](../postulates/0354-remote-free-mixed-lifetime-trace.md)
claimed the sharded chunk-publish advantage from commanded cycles survives
a realistic mixed request-lifetime trace with continuous frees, expecting
at least a 10 percent win at four workers.

## Change

Added `crates/locus-alloc/benches/remote_free_mixed_lifetime_trace.rs`.
One benchmark iteration runs a full deterministic serving trace:

- 64 requests arriving 4 per step; each allocates a 16-block prefill
  burst on arrival then 1 real 4 KiB block per decode step;
- decode lengths 16, 32, or 48 steps by request index; every fourth
  request cancels early after 8 decode steps;
- completed or cancelled requests dispatch their whole block vector
  round-robin to 4 persistent worker threads, which free immediately
  through the configured path with no per-cycle rendezvous;
- owner drains once per step and under pool backpressure; pool is 4096
  blocks so the 2688-block trace peak always fits;
- paths: shared_per_handle (one capacity-1024 queue, batch 64, all
  workers enqueue single handles) versus sharded_chunk (one capacity-8
  Vec queue per worker, one enqueue per completed request).

Each trace allocates and frees exactly 2688 blocks; the pre-benchmark
sample runs 4 traces asserting allocated == freed (10752) and every queue
balanced with pending 0.

## Host Validation

```bash
cargo bench -p locus-alloc --bench remote_free_mixed_lifetime_trace -- --sample-size 20 --warm-up-time 1 --measurement-time 3   # run twice
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

Counters in both runs: `allocated=10752 freed=10752` per sample, all
queues balanced, zero disconnects.

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `kv_remote_free_mixed_lifetime_shared_per_handle_w4_64req` | 280.82 us to 283.49 us | 281.60 us to 285.40 us |
| `kv_remote_free_mixed_lifetime_sharded_chunk_w4_64req` | 79.743 us to 80.715 us | 80.872 us to 81.381 us |

Raw logs: scratchpad `remote_free_mixed_lifetime_run1.log` and
`remote_free_mixed_lifetime_run2.log`. Criterion change estimates between
runs were under 1.1 percent on both cases.

## Interpretation

The postulate survives by a far larger margin than predicted: 3.5x
(282 us vs 80 us), reproduced within 1 percent across runs, versus the
1.9x seen in commanded cycles. Two honest caveats and one explanation:

1. This compares the combined design (sharded plus chunk) against the
   combined naive design (shared plus per-handle), matching the design
   decision from 0353, not isolating either lever. The commanded-cycle
   experiments already isolated them individually.
2. The gap is larger under the trace because the shared per-handle path
   pays its per-send cost 2688 times per trace while the owner is
   simultaneously trying to allocate, so worker enqueues and owner drains
   interleave badly; the chunk path does about 64 sends per trace and
   returns blocks in request-sized groups, which also gives the pool
   free-list better locality on subsequent allocations.
3. Continuous free is the shape that favors this design most, and it is
   the realistic one. The rendezvous-heavy cycles of 0351 to 0353
   understated the benefit, not overstated it.

Design consequence: the sharded chunk-publish remote-free path is
confirmed as the locus design under realistic churn, not just bursts.
The mixed-lifetime trace is now the standard substrate for allocator
comparisons.

## Next Question

On this same trace, how do jemalloc and mimalloc perform when the block
pool is replaced by raw 4 KiB heap allocations freed directly on worker
threads (their native cross-thread free path), versus the locus pool with
sharded chunk remote free? That is the priority-2 baseline comparison.
