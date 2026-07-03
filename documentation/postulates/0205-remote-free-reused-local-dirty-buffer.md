# Postulate 0205: Remote-Free Reused Local Dirty Buffer

Date: 2026-07-03

## Claim

Keeping worker-local dirty-owner buffers alive across service windows can
preserve before-collection flush correctness while avoiding per-window buffer
allocation churn in the measured runtime service-window sequence.

## Rationale

Experiments 0211 and 0212 showed that earlier fixed flush cadences add tracker
work and do not beat before-collection local flushing for the current
owner-window shape. The next lower-risk improvement is lifecycle rather than
cadence: a worker-owned buffer should normally survive across many enqueue and
collection windows.

The current benchmark creates a fresh local buffer for each collected owner
window. That is a conservative shape, but it may overstate allocation cost for
a production worker that can keep its local dirty buffer capacity after each
flush.

## Experiment

Add a real allocation benchmark path that:

- keeps one shared dirty-owner tracker for the whole service-window sequence;
- keeps per-owner local dirty buffers alive across windows;
- marks local dirty owners only after successful enqueue attempts;
- flushes the relevant local buffer immediately before tracked dirty
  service-window collection;
- proves each reused buffer keeps capacity after flushes;
- compares the reused-buffer path against fresh before-collection local
  flushing and direct dirty-enqueue tracker marking.

## Falsification

The postulate fails if reused buffers change allocation counters, lose tracker
generations, collect the wrong dirty owner, fail to retain local buffer
capacity after flushes, or measure slower than fresh before-collection local
flushing in the same real allocation sequence.

## Expected Value

If the postulate survives, Locus will have evidence that worker-owned local
dirty buffers should be long-lived and flushed on service demand, not recreated
for every service-window collection.
