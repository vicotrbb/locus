# Postulate 0352: Sharded Remote-Free Queues Remove Producer Contention

Date: 2026-07-04

## Statement

The producer-count penalty measured in experiment 0351 (roughly 5.3 us at one
producer versus 21 us at four producers through one shared bounded queue) is
dominated by send-side contention on the shared channel, not by drain-side
work. Giving each producer its own bounded remote-free queue, with the owner
draining all shards round-robin, should recover most of the penalty: p4
sharded should land materially closer to the p1 shared baseline than to the
p4 shared result. If instead drain-side aggregation across four queues costs
as much as the contention it removes, sharding is not the right lever and
the design should look at batching on the producer side instead.

## Rationale

Experiment 0351 proved producer count dominates batch limit under concurrent
enqueue and drain: p1 5.3 us, p2 9.6 us, p4 21 us for the same 256 blocks.
The shared `RemoteFreeQueue` is a bounded mpsc channel, so all producers
serialize on its send side. Sharding one queue per producer makes every
channel spsc-shaped, which removes producer-producer cache-line contention
entirely. The open cost question is the owner side: draining four queues
means four times the polling and stats overhead per cycle, plus worse drain
locality. This experiment measures which effect wins, which directly decides
whether the allocator's remote-free path should be sharded per producer (or
per node) in the serving design.

## Experiment

Add a `remote_free_sharded` benchmark to `locus-alloc` that reuses the 0351
shape (persistent producer threads, real `KvBlockPool` allocations, 256 real
4 KiB blocks per cycle, concurrent enqueue and drain) and compares, at
producer counts 2 and 4:

- shared: one `RemoteFreeQueue` of capacity 256, all producers on one sink;
- sharded: one capacity-(256/p) queue per producer, each producer enqueues
  only into its own queue, owner drains all shards round-robin until all
  256 blocks are freed.

Batch limit fixed at 64 for all cases, since 0351 showed batch limit is not
a lever under concurrency. Pre-benchmark sample cycles assert per-queue
submitted == drained and pending == 0 so no items are lost. Run the full
Criterion suite twice per the concurrency rule.

## Expected Result

Sharded p4 should beat shared p4 clearly (expected at least 1.5x) and land
near shared p2 or better. If sharded p4 is within noise of shared p4, the
penalty is drain-side or scheduler-bound and sharding is falsified as the
lever.
