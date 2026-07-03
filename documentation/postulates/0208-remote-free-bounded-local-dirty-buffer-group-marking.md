# Postulate 0208: Remote-Free Bounded Local Dirty Buffer Group Marking

Date: 2026-07-03

## Claim

A bounded local dirty-buffer group marking API can reject invalid or extremely
sparse owner IDs before vector growth while preserving the measured local
dirty-buffer lifecycle for validated service loops.

## Rationale

Experiment 0215 showed that integrated collection can validate owner
registration before flushing a vector-indexed local dirty buffer. Direct
marking still exposes a lower-level path: `mark_dirty` and `local_marker`
resize storage from the owner index. That is appropriate for trusted hot paths
that receive IDs from the owner registry, but it is not an adequate boundary
for paths that may see externally supplied, stale, or corrupted owner IDs.

The group needs an explicit bounded API that checks the owner index against a
caller-provided owner count before allocating local buffer storage. The hot
unbounded methods can remain available for already-validated paths, while
callers that need a safety boundary can choose the bounded variants.

## Experiment

Add fallible bounded methods that:

- accept an owner ID and an owner limit;
- reject owner IDs with indexes greater than or equal to the limit;
- do not grow local buffer storage on rejection;
- support both one-shot local marking and borrowed local markers;
- preserve duplicate-mark and flush counters for accepted owner IDs;
- are compared against the existing local marker path in the real allocation
  service-window sequence.

## Falsification

The postulate fails if rejected owner IDs allocate local buffer storage, if
accepted IDs change local duplicate or flush counters, if integrated collection
counters change, or if the bounded validated path adds enough overhead to make
it unsuitable for any measured service-loop use.

## Expected Value

If the postulate survives, Locus will have a production-facing safety boundary
for local dirty-buffer group marking while keeping the unbounded marker path
available for the tightest trusted hot loops.
