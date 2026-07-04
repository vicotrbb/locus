# Experiment 0355: Malloc Baselines on the Mixed-Lifetime KV Trace

Date: 2026-07-04

## Postulate

[Postulate 0355](../postulates/0355-mixed-lifetime-malloc-baselines.md)
claimed the locus pool with sharded chunk remote free (80 us per trace,
experiment 0354) beats jemalloc, mimalloc, and the system allocator
running the same trace with native cross-thread frees, with mimalloc
expected closest at 1.2x to 2x behind.

## Change

Added a shared trace module
`crates/locus-alloc/benches/mixed_lifetime_malloc/trace.rs` plus three
bench binaries with distinct global allocators:

- `mixed_lifetime_jemalloc` (`tikv_jemallocator::Jemalloc`);
- `mixed_lifetime_mimalloc` (`mimalloc::MiMalloc`);
- `mixed_lifetime_system` (macOS default).

The trace shape is identical to 0354 (64 requests, 4 arrivals per step,
16-block prefill, one 4 KiB block per decode step, decode lengths
16/32/48, every fourth request cancels after 8 steps, 2688 blocks per
trace). Blocks are `Vec<u8>` built with `Vec::with_capacity(4096)` plus a
first-byte write; completed requests dispatch their vectors to 4
persistent workers that drop them on the worker thread, the allocator's
native cross-thread free path. An `AtomicUsize` counts worker frees and
each iteration blocks until freed == allocated == 2688.

## Host Validation

```bash
cargo bench -p locus-alloc --bench mixed_lifetime_jemalloc -- --sample-size 20 --warm-up-time 1 --measurement-time 3   # likewise mimalloc, system; all run twice
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

Samples in every run: `allocated=10752 freed=10752` over 4 traces.

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `kv_mixed_lifetime_jemalloc_w4_64req` | 229.56 us to 230.49 us | 230.77 us to 237.75 us |
| `kv_mixed_lifetime_mimalloc_w4_64req` | 199.98 us to 204.21 us | 197.71 us to 198.91 us |
| `kv_mixed_lifetime_system_w4_64req` | 252.25 us to 278.31 us | 226.29 us to 231.68 us |
| locus sharded chunk (0354, same host, same trace shape) | 79.743 us to 80.715 us | 80.872 us to 81.381 us |

Raw logs: scratchpad `mixed_lifetime_malloc_run1.log` and
`mixed_lifetime_malloc_run2.log`.

## Interpretation

The postulate survives with the predicted ordering, by a larger margin
than expected:

1. Locus sharded chunk at 80 us beats mimalloc (198 to 204 us) by about
   2.5x, jemalloc (230 to 238 us) by about 2.9x, and the system allocator
   (226 to 278 us, the noisiest case) by about 3x.
2. mimalloc is indeed the closest competitor, consistent with its sharded
   free-list design being the same idea locus applies, but its
   general-purpose machinery (size-class lookup, page metadata, deferred
   remote-free segments) costs 2.5x on this fixed-size churn.
3. Honest caveat: the locus pool amortizes mapping and first-touch
   outside the measurement because blocks are pre-mapped and recycled;
   the malloc baselines pay allocator metadata work per operation but
   also benefit from thread caches recycling the same 4 KiB size class.
   Both are steady-state fair for a serving engine that owns a fixed KV
   budget, which is the target scenario.
4. Interestingly, the shared per-handle locus path from 0354 (282 us) is
   slightly worse than plain jemalloc. A badly shaped ownership handoff
   erases the entire benefit of pooling; the win comes specifically from
   sharding plus chunking, not from pooling alone.

Design consequence: the pooled sharded chunk remote-free design is
validated against the strongest general-purpose baselines on realistic
churn. Point 4 is the sharpest lesson for serving engines: a KV block
pool with a naive shared free queue is slower than just using jemalloc.

## Next Question

The 80 us locus figure still spends most of its time in owner-side drain
and pool free-list pushes done one handle at a time. Can the pool accept
a spliced chunk free (free a whole request's blocks as one linked segment
in O(1) or O(chunk) with no per-handle branching), and does that cut the
trace below 60 us? This tests whether chunk identity should be preserved
end to end, producer to free list, which no general-purpose allocator
does for caller-defined groups.
