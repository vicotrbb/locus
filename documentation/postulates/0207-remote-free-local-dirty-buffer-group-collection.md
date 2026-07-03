# Postulate 0207: Remote-Free Local Dirty Buffer Group Collection

Date: 2026-07-03

## Claim

Integrating local-buffer flush and tracked dirty collection into one
production-facing owner-registry method can remove caller-side collection
plumbing without changing correctness and without measurable overhead versus
the existing local dirty-buffer group path.

## Rationale

Experiment 0214 moved the shared tracker and reusable per-owner local buffers
into `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers`, but callers still need
to flush one owner buffer, pass the group's tracker into
`collect_tracked_dirty_service_window`, and reason about tracker clearing after
success. That leaves the service path correct but more exposed than necessary.

The owner registry already owns the tracked dirty collection semantics. It is
the right boundary to combine one local owner flush with one tracked dirty
collection pass while returning the local flush counters for observability.

## Experiment

Add a method that:

- accepts `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers` and one owner ID;
- flushes that owner's local buffer into the group's tracker;
- collects through the existing tracked dirty service-window path;
- clears only the captured tracker snapshot after success;
- leaves tracker marks available after collection errors;
- returns both window stats and local flush stats;
- matches the existing local buffer group counters and timing envelope.

## Falsification

The postulate fails if the integrated method loses tracker generation safety,
clears local or shared dirty state incorrectly on errors, changes real
allocation counters, or measures slower than the existing local buffer group
path in the same allocation sequence.

## Expected Value

If the postulate survives, Locus will have a smaller and harder-to-misuse
production service-window API for the currently measured local dirty-buffer
lifecycle.
