# The Locus research record: a reader's guide

This directory is the complete, immutable research record behind the
`locus` crate. No file in `experiments/`, `postulates/`,
`evaluations/`, or `dev-notes/` is ever edited after the fact,
including filenames; falsified results ship unedited. This guide is
the only navigational layer on top.

## How the methodology works

Every change starts as a numbered postulate in `postulates/`: a
falsifiable prediction written before any code or measurement. The
matching experiment in `experiments/` (same number) records the
harness, two benchmark runs (a third run when the first two disagree
by more than 5 percent), and a verdict against the prediction.
Falsified postulates are kept and labeled, not rewritten; the
prediction being wrong is data. Standing suites live in
`evaluations/` and are frozen once published: LOCUS-EVAL v1 can gain
an addendum (it has one) but never an edited table. Raw Criterion
logs that survived their sessions are preserved verbatim in
`evaluations/logs/` (see `dev-notes/2026-07-05-evidence-rescue.md`
for exactly which raw logs survived; for experiments 0001 to 0350 the
tables inside the experiment documents are the only record).
Cross-cutting decisions are ADRs in `adr/`; running syntheses are
dated notes in `dev-notes/`; the external grounding corpus is in
`research/`.

## The main validated thread

The published crate rests on this chain, in order:

1. 0351 (`experiments/0351-remote-free-concurrent-producer-contention.md`):
   real concurrent producers collapse the single shared bounded
   queue; single-threaded microbenchmarks had hidden the contention.
2. 0352: sharding producer queues recovers some loss but keeps the
   tuning surface.
3. 0353: publishing whole chunks instead of per-handle entries
   changes the transport economics decisively.
4. 0354: a deterministic mixed-lifetime KV trace (the workload shape
   a serving engine actually produces) confirms the chunk path.
5. 0355: malloc baselines (jemalloc, mimalloc, system) on the same
   trace; pooling with a bad handoff loses to every malloc.
6. 0356 (falsified, and kept): chunk-preserving pool recycling gained
   nothing and regressed the base path; the pool keeps its flat LIFO
   free list.
7. 0357: lock-free per-worker chunk mailboxes beat bounded chunk
   queues and delete the entire capacity/retune tuning surface. This
   is the design the crate ships.
8. 0358: LIFO reuse order wins once blocks are actually written
   (cache warmth measured, not assumed).
9. 0359: mapped-region backing for the KV pool prices the host cost
   and the Linux bind path.
10. Postulate 0360 plus `evaluations/0001-locus-eval-v1.md`:
    LOCUS-EVAL v1, the standing four-workload scoreboard against
    jemalloc, mimalloc, and system malloc, with predicted-vs-observed
    rankings and per-workload verdicts.
11. The audit addendum inside the same evaluation document
    (post-publication section): a touch-parity probe found the
    untouched-trace margins overstated (2.3x/3.9x/4.6x untouched
    compress to 1.6x/2.7x/2.3x over mimalloc at touch parity), plus
    one latent harness bug (`allocate_with_backpressure` discards
    drain counts) that provably did not affect v1 results and is
    documented for v2. Quote margins from the addendum, not the raw
    v1 tables.

## Dead threads, and what killed them

- Bounded-queue capacity/retune/telemetry apparatus (experiments
  0059, 0142, 0143, 0146 and a long tail of tuning work): obsoleted
  wholesale by 0357, whose design has no capacity to tune. The code
  was deleted in the LOCUS-OSS reorganization; see
  `dev-notes/2026-07-05-code-graveyard.md` for recovery hashes.
- The remote-free service telemetry/rollup thread (experiments
  roughly 0335 to 0350, with intentionally unwieldy repeated-suffix
  filenames): layer upon layer of JSON rollup validation machinery
  that never produced an allocator improvement. It dead-ended; 0351's
  turn to real concurrency made it moot. The filenames are preserved
  as-is; they are an honest record of a runaway abstraction spiral,
  and the graveyard note maps what was deleted.
- Chunk-preserving pool recycling: falsified directly by 0356.
- Earlier exploratory threads (scratch arenas, THP probing, pinned
  memory, placement validation gates, experiments 0001 to 0350):
  infrastructure and evidence-reader work that the validated thread
  stands on; none of it was falsified, but only the KV-pool and
  mailbox results carry the crate's performance claims. The code
  survives as internal (`doc(hidden)`) modules and the unpublished
  locus-observe and locus-validate crates.

## ADRs

- ADR 0001: explicit domain runtime; no global allocator replacement.
- ADR 0002: the narrow unsafe boundary, originally the locus-sys
  crate, amended 2026-07-05 when the boundary became the `sys` module
  of the merged crate (guarantee unchanged).
- ADR 0003: validation gates live in a separate unpublished crate.

## Reading order for a newcomer

Read `evaluations/0001-locus-eval-v1.md` end to end (including the
audit addendum), then 0357, then 0351 for why the obvious design
fails, then the graveyard and evidence-rescue dev-notes for how the
record maps to the shipped code.
