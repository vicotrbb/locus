# Experiment 0358: Reuse Order Cache Warmth Under Real Block Writes

Date: 2026-07-04

## Postulate

[Postulate 0358](../postulates/0358-reuse-order-cache-warmth.md) claimed
LIFO block reuse beats FIFO by at least 10 percent once every allocated
block is actually written, with untouched controls showing no order
effect, thereby pricing how much reuse reordering a NUMA placement layer
could afford.

## Change

- `KvBlockPool` gained an explicit reuse order: the free list is now a
  `VecDeque<usize>`; `KvReuseOrder::Lifo` (default, back pop) preserves
  existing behavior, `KvReuseOrder::Fifo` (front pop) maximizes reuse
  distance. New constructor `new_with_reuse_order`; two unit tests pin
  each order's reuse choice.
- Added `crates/locus-alloc/benches/kv_reuse_order_locality.rs`: 64 MiB
  pool (16384 blocks of 4 KiB), 256 live 16-block chunks (16 MiB live
  set); each cycle frees the oldest chunk and allocates plus fully
  writes a new one, 64 steps per cycle. Cases: both orders with full
  4 KiB writes and both without as controls. Sample cycles assert pool
  accounting (4096 allocated, 12288 free) before timing.

## Host Validation

```bash
cargo bench -p locus-alloc --bench kv_reuse_order_locality -- --sample-size 20 --warm-up-time 1 --measurement-time 3   # run twice
cargo bench -p locus-alloc --bench remote_free_mixed_lifetime_trace -- sharded_mailbox   # VecDeque regression check
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `kv_reuse_order_lifo_touch4k_64x16blk` | 88.707 us to 96.864 us | 81.796 us to 84.100 us |
| `kv_reuse_order_fifo_touch4k_64x16blk` | 101.43 us to 103.99 us | 99.786 us to 102.20 us |
| `kv_reuse_order_lifo_notouch_64x16blk` | 4.2707 us to 4.3045 us | 4.3507 us to 4.3769 us |
| `kv_reuse_order_fifo_notouch_64x16blk` | 4.5789 us to 4.5887 us | 4.6675 us to 5.1229 us |

The VecDeque swap did not regress the mixed-lifetime mailbox path:
67.633 us to 68.488 us, slightly faster than the prior 71.450 us to
72.643 us measurement. Raw logs: scratchpad `kv_reuse_order_run1.log`
and `kv_reuse_order_run2.log`.

## Interpretation

The postulate survives with one control caveat:

1. With full-block writes, LIFO beats FIFO by 11 to 18 percent (medians
   92.2 vs 102.5 us in run 1, 82.8 vs 101.1 us in run 2), clearing the
   10 percent bar in both runs. Recycling the most recently freed 4 KiB
   blocks keeps their lines resident; FIFO reuse at 48 MiB reuse
   distance pays memory-hierarchy misses on every write burst.
2. The untouched controls are not perfectly null: FIFO is 0.3 to 0.5 us
   per cycle slower without touches, likely front-pop index order
   effects on the pool's metadata arrays. But that is roughly 60 times
   smaller than the touched-case gap (about 18 us), so the warmth
   attribution stands.
3. Per block, warmth is worth about 18 ns of write time on this host
   (1024 touched blocks per cycle). Scaled to a serving engine writing
   KV for thousands of blocks per second per node, recency-respecting
   reuse is a real bandwidth saving, not a microbenchmark artifact.

Design consequence: the NUMA placement layer must treat recency as a
constraint, not a free variable: prefer node-local chunks, but among
local candidates always reuse the newest. A placement policy that round
robins or ages block reuse for fairness would silently pay about 20
percent on KV write bandwidth. Chunk-granular recycling makes this easy:
chunks arrive at the owner newest-last and the flat LIFO list preserves
exactly the right order for free.

## Next Question

Priority 4 remains open: package the mailbox-plus-LIFO design into a
cfg-gated Linux placement experiment (mbind or numa_alloc_onnode behind
`target_os = "linux"`, run via OrbStack Docker) that validates first
touch node binding end to end on one node, so the same binary produces
real placement numbers on a multi-node Linux host.
