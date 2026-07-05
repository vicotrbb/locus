# Postulate 0360: LOCUS-EVAL v1 Predicted Rankings

Date: 2026-07-05

## Statement

A fixed four-workload evaluation suite (LOCUS-EVAL v1) run over five
contenders (locus-mailbox, locus-shared, jemalloc, mimalloc, system
malloc) will confirm the chunk-mailbox design on churn-shaped work but
its advantage will shrink, and may invert on quality, on the long-tail
workload, because eight 512-step requests hold blocks far longer than
the reuse distance the LIFO list is tuned for and dilute the value of
chunk-shaped handoff (fewer, larger frees relative to total work).

## Predicted rankings (velocity, fastest first)

1. steady-decode: locus-mailbox, then mimalloc, jemalloc, system,
   locus-shared. Gap locus-mailbox over mimalloc about 2.5x to 3x
   (0355/0357). If any baseline beats locus-mailbox here, that
   contradicts 0355/0357 and is presumed a harness bug until
   investigated.
2. burst-storm: same order. The mailbox should widen its lead slightly:
   mass arrival plus mass cancellation is exactly the chunk-shaped free
   pattern, while the malloc thread caches absorb the burst less well
   and locus-shared pays 2688 contended enqueues in a tighter window.
3. long-tail: same order but with the locus-mailbox lead over mimalloc
   shrinking, predicted under 2x. Long-lived requests mean fewer
   completions per step, so the transport (the locus advantage) is a
   smaller share of trace time; allocation-path costs dominate and the
   pool's allocation path is only modestly cheaper than a warmed
   mimalloc thread cache. This is the workload named most likely to
   embarrass the design.
4. churn-touch: locus-mailbox, mimalloc, jemalloc, system,
   locus-shared, but with compressed gaps (predicted 1.2x to 1.6x over
   mimalloc) because every contender pays the same 4 KiB write
   bandwidth per block, which dominates the loop (0358 showed touch
   cost is about 20x the bookkeeping cost).

## Predicted quality and stability

- Quality (peak outstanding blocks vs theoretical live peak): the
  locus paths should sit closest to 1.0 on steady-decode and
  burst-storm because the owner drains every step. On long-tail the
  ratio should stay near 1.0 for everyone (frees are rare). The malloc
  baselines cannot be worse than about 1.1 on any workload since
  workers free promptly; if locus-shared shows a high ratio under
  burst-storm, that is queue backlog and it ships as a finding.
- Stability (Criterion interval width): locus-mailbox tightest
  (0357 showed sub-1-percent intervals), system malloc widest (0355
  showed 10 percent swings).
- Proof gates (allocated == freed, zero lost blocks) must pass for all
  contenders on all workloads; any failure is recorded as unsupported,
  not skipped.

## Experiment

Implement the suite as benches under `crates/locus-alloc/benches` with
one shared workload-definition module; run every contender twice
(third run when the first two medians disagree by more than 5
percent); write `documentation/evaluations/0001-locus-eval-v1.md` with
the full workload x contender x metric table, predicted vs observed
rankings, and per-workload verdicts. Workloads: steady-decode (0354
trace), burst-storm (all 64 requests arrive at step 0), long-tail
(8 requests of 512 decode steps, 56 of 16), churn-touch (0358
write-touch churn routed through each contender's free path).

## Expected Result

locus-mailbox wins steady-decode and burst-storm by 2x or more,
churn-touch by under 1.6x, and long-tail by under 2x, with locus-shared
last or second-to-last everywhere. The most falsification-rich outcome
would be mimalloc taking long-tail outright; that result, if it occurs,
ships with equal prominence and bounds the design's applicability to
short-request-dominated serving.
