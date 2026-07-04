# Remote-Free Concurrency Lessons (Experiments 0351 to 0353)

Date: 2026-07-04

What an LLM serving engine could use from this thread today:

1. Do not tune remote-free batch limits from single-threaded sweeps. The
   6.6x large-batch advantage from experiment 0059 vanishes entirely once
   drain overlaps production (0351). Any KV-cache free-path tuning must be
   validated with real concurrent producer threads.
2. Producer count is the cost driver. Through one shared bounded queue,
   the same 256-block free cycle costs 5.3 us at 1 producer, 9.6 us at 2,
   21 us at 4 (0351).
3. Shard remote-free queues per producer. One spsc-shaped queue per
   producer with round-robin owner drain cuts p4 from 21 us to 12.8 us,
   a reproduced 1.63x, with no loss at any other point (0352).
4. Publish frees in chunks, not per handle. A batch-publish ABI (one Vec
   per producer per burst) saves a further roughly 2 us per cycle at any
   producer count, taking p4 to 10.8 us; combined with sharding that is
   1.94x over the naive shared design (0353).
5. The remaining floor is scheduler coordination, roughly 1.8 us per
   additional producer thread on this host (5.3 / 7.3 / 10.8 us at p1 /
   p2 / p4 with both optimizations). Channel engineering beyond this point
   is wasted; the next win must come from avoiding synchronous rendezvous,
   which matches how real serving engines free continuously anyway.

Design decision recorded: the locus remote-free path will expose sharded
per-producer queues with a chunked publish ABI. Next thread (priority 2)
replaces commanded all-at-once cycles with a realistic mixed-lifetime KV
trace and adds jemalloc and mimalloc baselines under the same trace.
