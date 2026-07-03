# Postulate 0201: Remote-Free Enqueue Dirty Owner Marks

Date: 2026-07-03

## Claim

A cloneable dirty-marking remote-free sink can mark an owner dirty on each
successful enqueue while preserving queue semantics and dirty-owner collection
correctness.

## Rationale

Experiment 0208 added dirty-owner collection, but callers still had to mark
owners manually. A live service loop should be able to hand producers a normal
remote-free enqueue handle that also records owner activity. Direct enqueue
marking is the earliest signal that an owner has new remote-free work, but it
must not lose marks that arrive while a service window is collecting.

## Experiment

Add an enqueue-side dirty mark path that:

- wraps `RemoteFreeSink` without changing its existing API behavior;
- marks the owner only after a successful enqueue or try-enqueue;
- does not mark on full or disconnected enqueue attempts;
- records dirty marks in a shared tracker;
- captures dirty-owner snapshots with per-owner generations;
- clears only snapshot generations that were successfully collected;
- preserves newer marks that arrive during collection.

Benchmark the dirty-marking sink with real runtime-collected owner windows that
cover a confirmed owner, a rolled-back owner, and a mutation-limited owner.

## Falsification

The postulate fails if enqueue marking changes queue counters, marks owners on
failed enqueue attempts, loses concurrent marks during snapshot clearing, or if
real allocation counters diverge from the dirty-owner collection benchmark.

## Expected Value

If the postulate survives, Locus will have a reusable live-runtime signal path
from remote enqueue handles to dirty-owner service-window collection.
