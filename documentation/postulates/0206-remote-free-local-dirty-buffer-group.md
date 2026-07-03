# Postulate 0206: Remote-Free Local Dirty Buffer Group

Date: 2026-07-03

## Claim

A production-facing local dirty-buffer group can own the shared dirty-owner
tracker and reusable per-owner local buffers without adding measurable overhead
to the reused-buffer service-window path.

## Rationale

Experiment 0213 showed that long-lived worker-local buffers with
service-demand flushing beat both fresh local buffers and direct dirty-enqueue
tracker marking in the current real allocation sequence. That evidence still
lives partly in benchmark-only scaffolding. Production callers should not need
to manually pair one tracker with a separate indexed buffer vector.

A helper that owns both pieces can make the measured lifecycle harder to use
incorrectly: successful enqueue paths mark a local buffer, service collection
flushes the owner buffer into the helper's tracker, and collection uses the
same tracked dirty service-window path.

## Experiment

Add a reusable helper that:

- owns one `RemoteFreeServiceRuntimeDirtyOwnerTracker`;
- owns reusable local buffers indexed by `RemoteFreeServiceRuntimeOwnerId`;
- marks owners dirty locally without touching the shared tracker;
- flushes one owner buffer into the shared tracker on demand;
- retains local buffer capacity after flush;
- exposes the tracker for tracked dirty service-window collection;
- matches the benchmark-only reused-buffer counters and timing envelope.

## Falsification

The postulate fails if the helper loses tracker generation safety, flushes the
wrong owner, fails to retain local buffer capacity after flush, changes real
allocation counters, or measures slower than the benchmark-only reused-buffer
path in the same allocation sequence.

## Expected Value

If the postulate survives, Locus will have a production API for the best
measured local dirty-buffer lifecycle instead of benchmark-only lifecycle
scaffolding.
