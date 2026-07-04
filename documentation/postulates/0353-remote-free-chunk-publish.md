# Postulate 0353: Chunk Publish Cuts the Remote-Free Coordination Floor

Date: 2026-07-04

## Statement

The roughly 12.8 us floor left at four producers after queue sharding
(experiment 0352) is dominated by per-handle enqueue work: each producer
performs 64 individual channel sends per cycle, so 256 sends cross thread
boundaries even though contention is gone. Publishing each producer's whole
chunk as a single queue item (one send of a `Vec<KvBlockHandle>` per
producer per cycle, 4 sends total) should cut the p4 sharded cycle time
materially, below 10 us. If chunk publish does not beat per-handle sharded
enqueue, the floor is thread wake-up and scheduling, not enqueue path cost,
and further channel engineering is pointless on this host.

## Rationale

Experiment 0352 showed sharding removes the contention term (21 us to 12.8
us at p4) but leaves a floor far above the p1 baseline of 5.3 us, and p2
sharded barely beat p2 shared. Two candidate explanations remain:

- per-item cost: 256 individual bounded-channel sends and drains per cycle,
  each with its own atomic sequence and stats update;
- coordination cost: waking four producer threads and rendezvousing with
  the owner, largely independent of item count.

Chunk publish collapses 256 sends into 4 while keeping the same threads,
the same real block handles, and the same pool free path, so it isolates
the per-item term cleanly. The answer decides whether the allocator's
remote-free ABI should accept handle batches (like mimalloc's free-list
splicing) instead of single handles.

## Experiment

Extend the sharded benchmark family with `remote_free_chunk_publish`
benchmarks at p2 and p4, comparing under identical cycle shape:

- per-handle: one capacity-(256/p) `RemoteFreeQueue<KvBlockHandle>` per
  producer, 256/p sends each (the 0352 sharded winner);
- chunk: one capacity-2 `RemoteFreeQueue<Vec<KvBlockHandle>>` per producer,
  each producer sends its whole chunk as one item, owner drains vectors and
  frees every contained handle into the pool.

Counters must prove per-queue submitted == drained and, for the chunk case,
that total freed handles equal 256 per cycle. Two full Criterion runs.

## Expected Result

Chunk publish at p4 should land clearly below per-handle sharded (12.8 us),
ideally near or below 10 us. p2 should show a smaller but same-direction
gain. If both land within noise of per-handle, the coordination-floor
explanation wins and the postulate is falsified.
