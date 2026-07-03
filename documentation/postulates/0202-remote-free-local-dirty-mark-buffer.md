# Postulate 0202: Remote-Free Local Dirty Mark Buffer

Date: 2026-07-03

## Claim

A small per-worker dirty-owner buffer can batch successful enqueue marks before
touching the shared dirty-owner tracker, preserving dirty-owner collection
correctness while reducing enqueue-side shared synchronization in the measured
service-window path.

## Rationale

Experiment 0209 proved that direct enqueue-side dirty marking works, but the
cloneable dirty sink touches a mutex-backed tracker after every successful
enqueue. A service worker usually knows the owner being submitted to during a
burst. In that shape, a local buffer can deduplicate repeated owner marks and
flush only the unique owner IDs into the shared tracker before service-window
collection.

The local buffer should stay compact because it is expected to hold a tiny set
of owners per worker turn. A Vec-only first-marked-order buffer is a better fit
than a tree-backed set for this path until a real benchmark shows otherwise.

## Experiment

Add a local dirty-owner mark buffer that:

- records unique owner IDs in first-marked order;
- counts duplicate local marks that were deduplicated before flush;
- flushes unique owner IDs into `RemoteFreeServiceRuntimeDirtyOwnerTracker`;
- reports the number of flushed owners and newly pending tracker marks;
- clears local marks after a flush;
- preserves tracker snapshot generation safety when an owner was already
  pending in the shared tracker.

Benchmark the local-buffer path with the same real allocation service-window
sequence used by the direct dirty-enqueue tracker path.

## Falsification

The postulate fails if the local buffer marks failed enqueue attempts, changes
allocation or service-window counters, loses tracker marks across snapshot
clearing, or benchmarks slower than direct tracker marking in the same real
allocation sequence.

## Expected Value

If the postulate survives, Locus will have a low-contention dirty-owner marking
path for worker-owned enqueue loops while keeping the shared tracker as the
service collection boundary.
