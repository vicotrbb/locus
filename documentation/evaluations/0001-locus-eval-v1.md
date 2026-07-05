# LOCUS-EVAL v1: Standing Allocator Scoreboard

Date: 2026-07-05
Postulate: [0360](../postulates/0360-locus-eval-v1.md)
Suite version: LOCUS-EVAL v1 (frozen). Any change to workloads,
metrics, or contenders requires LOCUS-EVAL v2; this document is never
edited to reflect a changed suite.

## Suite definition

Benches: `crates/locus-alloc/benches/locus_eval_locus.rs` plus
`locus_eval_{jemalloc,mimalloc,system}.rs`, sharing
`benches/locus_eval/workloads.rs` (the versioned unit) and
`benches/locus_eval/malloc_runner.rs`. Four workers free on real
threads in every case; no synthetic sleeps anywhere.

Workloads (all deterministic):

- steady-decode: the 0354 trace. 64 requests, 4 arrivals per step,
  16-block prefill, one 4 KiB block per decode step, decode lengths
  16/32/48, every fourth request cancels after 8 steps. 2688 blocks.
- burst-storm: same requests, all 64 arrive at step 0. 2688 blocks.
- long-tail: 4 arrivals per step; every eighth request decodes 512
  steps, the other 56 decode 16. 6016 blocks, peak live 4168.
- churn-touch: the 0358 churn shape routed through each contender's
  free path: 256 live 16-block chunks, 64 steps per cycle, each step
  frees the oldest chunk on a worker and allocates plus fully writes a
  new 16-block chunk (4 KiB per block).

Contenders, one config each: locus-mailbox (per-worker chunk
mailboxes, LIFO pool), locus-shared (one shared per-handle queue,
capacity 1024, batch 64), jemalloc, mimalloc, system malloc (macOS),
the last three freeing natively on worker threads.

Metrics: velocity is the Criterion median over two runs (a third run
when the first two disagree by more than 5 percent); stability is the
confidence-interval width; quality is peak outstanding blocks over the
theoretical instant-free live peak; proof gates (allocated == freed,
zero lost blocks, queues balanced) are hard gates. Overlapping
intervals are reported as tied.

Host: Apple Silicon (same host as 0354 to 0358). Raw logs: scratchpad
`locus_eval_{locus,jemalloc,mimalloc,system}_run{1,2}.log` plus
`locus_eval_{locus,system}_run3.log`.

## Results: velocity (Criterion median per trace, us)

| Workload | locus-mailbox | locus-shared | jemalloc | mimalloc | system |
| --- | ---: | ---: | ---: | ---: | ---: |
| steady-decode | 86.9 / 84.9 / 81.4 | 290.9 / 280.7 / 274.0 | 233.7 / 227.6 | 201.2 / 197.3 | 229.9 / 228.7 / 232.6 |
| burst-storm | 37.5 / 39.5 / 37.6 | 307.9 / 293.9 / 294.3 | 203.6 / 210.2 | 147.2 / 145.2 | 176.6 / 166.9 / 173.7 |
| long-tail | 66.4 / 61.5 / 60.4 | 646.1 / 622.5 / 635.2 | 414.3 / 418.5 | 283.0 / 278.6 | 464.4 / 466.4 / 443.1 |
| churn-touch | 170.0 / 183.8 / 168.6 | 238.0 / 236.9 / 230.8 | 223.0 / 229.0 | 201.6 / 209.0 | 181.2 / 202.1 / 196.1 |

Third runs were triggered by the 5 percent rule for locus-mailbox
(burst-storm, long-tail, churn-touch) and system (burst-storm,
churn-touch); the whole binary was rerun in each case, so the extra
medians are reported for every workload of those binaries.

## Results: quality (peak outstanding blocks / theoretical peak)

Theoretical peaks: steady-decode 1344, burst-storm 1537, long-tail
4168, churn-touch 4096. Worst observed ratio across runs:

| Workload | locus-mailbox | locus-shared | jemalloc | mimalloc | system |
| --- | ---: | ---: | ---: | ---: | ---: |
| steady-decode | 1.35 | 1.47 | 1.04 | 1.06 | 1.12 |
| burst-storm | 1.50 | 1.62 | 1.12 | 1.11 | 1.14 |
| long-tail | 1.01 | 1.01 | 1.01 | 1.01 | 1.01 |
| churn-touch | 1.01 | 1.01 | 1.01 | 1.01 | 1.00 |

Proof gates passed for every contender on every workload (allocated ==
freed each trace, mailbox submitted == drained, shared-queue counters
balanced, zero disconnects). No workload was unsupported by any
contender.

## Results: stability (confidence-interval width)

locus-mailbox intervals were under 2 percent in the majority of runs
but showed one wide long-tail interval (run 1: 61.3 to 73.3 us, about
18 percent) and one wide churn-touch interval (run 2, about 13
percent); the third runs were tight (under 2 percent). mimalloc and
jemalloc stayed under 4 percent everywhere. system malloc was the
least stable baseline (churn-touch medians spread 181 to 202 us across
runs, an 11 percent swing). locus-shared sat in the 2 to 7 percent
band. The 0351 lesson holds: concurrent variance is a real signal, and
the mailbox's occasional wide interval under long-tail warrants a
question in v2, not a rank change (its worst interval still beats the
best baseline median by 3.8x).

## Predicted vs observed ranking

| Workload | Predicted (0360) | Observed |
| --- | --- | --- |
| steady-decode | mailbox, mimalloc, jemalloc, system, shared | mailbox, mimalloc, jemalloc = system (tied), shared. Prediction correct except jemalloc/system tie. |
| burst-storm | mailbox, mimalloc, jemalloc, system, shared | mailbox, mimalloc, system, jemalloc, shared. System/jemalloc order inverted from prediction. |
| long-tail | same order, mailbox lead under 2x | mailbox, mimalloc, jemalloc, system, shared, with the mailbox lead GROWING to about 4.6x. Prediction wrong in direction. |
| churn-touch | mailbox, mimalloc, jemalloc, system, shared; gaps 1.2x to 1.6x | mailbox, system, mimalloc, jemalloc, shared; mailbox over system only about 1.15x. Two misses: system ranked second, and the gap fell below the predicted floor. |

## Verdicts per workload

- steady-decode: locus-mailbox survives at 2.3x over mimalloc (85 vs
  199 us), slightly under the predicted 2.5x to 3x; the suite harness
  (8192-block pool, peak tracking) reads a few microseconds above the
  0354 harness but no baseline came near, so the 0355/0357 sanity
  condition holds. Cost to a serving engine of the naive alternative:
  locus-shared is slower than every malloc here; pooling with a bad
  handoff is worse than no pooling.
- burst-storm: the design's best velocity case, 3.9x over mimalloc (38
  vs 146 us); mass cancellation is exactly chunk-shaped. The loss is
  quality: locus-mailbox transiently holds 1.50x the theoretical peak
  while every malloc stays at or under 1.14x, because the owner drains
  once per step while malloc workers free instantly. A serving engine
  sized to the theoretical KV peak would need roughly 50 percent
  headroom to absorb a full burst-cancel under this drain cadence, or
  a drain-on-allocate-failure fallback (which the pool already has and
  which bounds the overshoot at the pool size). This loss ships with
  the win.
- long-tail: the workload named most likely to embarrass the design
  instead produced its largest margin, about 4.6x over mimalloc (61 vs
  280 us). The postulate's mechanism was wrong: long requests do not
  dilute the transport advantage, they multiply allocations (6016 vs
  2688), and the pool's pop-an-index allocation path scales with
  allocation count far better than malloc thread caches holding a 16.5
  MiB live set. The 0360 risk prediction is falsified in the
  favorable direction; the honest caveat is that the trace still never
  reads or writes trace-workload block memory, so fragmentation-driven
  cache effects on long-tail are out of scope until a touched variant
  exists (next questions).
- churn-touch: the narrowest and most honest result. Once every block
  is fully written, the mailbox wins by only about 1.15x over system
  malloc (169 vs 196 us medians) and their intervals brush in one run
  pair; mimalloc and jemalloc fall behind system here, inverting the
  bookkeeping-only ranking. Write bandwidth dominates and allocator
  choice compresses toward a tie. Velocity claims made from untouched
  traces overstate the real-world gap roughly 3x on this host, which
  is exactly why this workload is in the suite. The design survives
  but a serving engine should expect the allocator win to be a
  single-digit percentage of end-to-end step time once KV writes are
  counted.

## Overall

locus-mailbox wins all four workloads on velocity; locus-shared is
last or second-to-last on all four, confirming the 0355 lesson that
pooling without the chunk-shaped handoff is worse than any malloc.
The two results that constrain the design story: burst quality
overshoot (1.5x transient footprint) and the churn-touch compression
(1.15x over system malloc when bandwidth is honest). Neither overturns
a 0357 design decision; both bound the claims a serving engine should
repeat.

## Next questions (deferred to LOCUS-EVAL v2, suite frozen)

1. A touched variant of the three trace workloads (write each block
   once on allocation) to price the untouched-trace overstatement per
   workload, not just in churn form.
2. Owner drain cadence sensitivity: does draining every allocation
   failure only (no per-step drain) change the burst-storm quality
   ratio, and at what velocity cost?
3. A long-tail variant with cancels among the 512-step requests, the
   worst case for chunk-size variance in mailboxes.
4. Rerun on Linux (OrbStack) to check whether system-malloc's strong
   churn-touch showing is a macOS large-allocation artifact.
