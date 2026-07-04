# Postulate 0357: Lock-Free Chunk Mailboxes Beat Bounded Queues for KV Remote Free

Date: 2026-07-04

## Statement

The bounded-channel protocol is now the dominant cost of the locus
remote-free path. Replacing each worker's `RemoteFreeQueue<Vec<..>>` with
a minimal lock-free chunk mailbox (a Treiber stack of request-chunk
nodes: producers CAS-push one heap node per completed request, the owner
takes the entire list with one atomic swap) removes capacity accounting,
per-item stats atomics, and rendezvous blocking from the free path, and
should cut the mixed-lifetime trace measurably below the 80 us
sharded-chunk-queue design. Combined with 0353 to 0356 this would
establish the innovative core of locus: request-chunk-granular,
mailbox-based remote free, distinct from both bounded-queue designs and
from per-allocation remote-free lists in mimalloc and snmalloc, which
batch by allocator page or size class rather than by caller-defined
request groups.

## Rationale

Experiment 0356 falsified pool bookkeeping as the bottleneck: the
remaining 80 us lives in the handoff machinery. `RemoteFreeQueue` pays a
bounded mpsc protocol (capacity check, occupancy accounting, submitted
and drained counters) per publish even in the chunk configuration. The
theoretical minimum for cross-thread ownership transfer is one atomic
CAS per publish and one swap per drain sweep. KV serving frees in
request-sized chunks a few times per step, so the per-publish constant is
paid rarely but sits on the owner's critical drain loop; making the drain
a single pointer swap per worker also removes the per-shard drain_batch
call overhead. Safety: the mailbox needs unsafe pointer code, kept
behind a narrow interface with stress tests, per project rules.

## Experiment

1. Add `ChunkMailbox<T>` to `locus-alloc` (new `remote_free/mailbox.rs`):
   `push(&self, T)` CAS-pushes a boxed node; `take_all(&mut FnMut(T))`
   swap-takes the list and delivers items in FIFO order after reversal;
   `Drop` reclaims leftover nodes. Unit tests: single-thread round trip,
   FIFO order per producer, multi-producer stress (4 threads times 10k
   chunks, all delivered exactly once), drop without drain leaks nothing
   (asserted via a drop-counting payload).
2. Add a `sharded_mailbox` path to the mixed-lifetime trace benchmark:
   one `ChunkMailbox<Vec<KvBlockHandle>>` per worker, workers push
   completed request chunks, owner take_alls each mailbox per step and
   frees per handle into the pool (identical pool ops to sharded_chunk).
   Trace accounting still proves allocated == freed == 2688.
3. Run the suite twice, comparing sharded_mailbox against sharded_chunk
   on the same runs.

## Expected Result

sharded_mailbox should beat sharded_chunk by at least 10 percent (below
about 72 us against the 80 us baseline). If it lands within noise, the
cost is not the queue protocol but the owner's polling cadence and
scheduler wake-ups, and the innovation claim shifts from mailbox
mechanics to the chunk-granular free ABI already proven in 0354/0355.
