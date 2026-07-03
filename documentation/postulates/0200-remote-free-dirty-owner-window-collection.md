# Postulate 0200: Remote-Free Dirty Owner Window Collection

Date: 2026-07-03

## Claim

An ordered dirty-owner set can avoid scanning every registered remote-free
owner on each service window while preserving the same guarded retune behavior
as explicit owner collection.

## Rationale

Experiment 0207 added borrow-scoped owner summary collection, but callers still
choose which owner IDs to collect. A live service loop should not need to scan
idle owners when remote producers can mark owners that received new work. A
small dirty-owner set can deduplicate those marks, preserve deterministic
collection order, and feed the existing collection helper.

## Experiment

Add a dirty-owner collection helper that:

- records marked owner IDs in first-marked order;
- deduplicates repeated owner marks inside one service window;
- exposes the number of pending dirty owners;
- collects only marked owners through `collect_service_window`;
- clears marks only after a successful collection window;
- preserves dirty marks when collection fails;
- preserves the shared service mutation budget across collected owners.

Benchmark the helper with real runtime-collected owner windows that cover a
confirmed owner, a rolled-back owner, and a mutation-limited owner.

## Falsification

The postulate fails if repeated owner marks cause duplicate collection, if
errors clear dirty marks before a retry can occur, if mutation budget becomes
owner-local, or if real allocation counters diverge from the full collection
benchmark.

## Expected Value

If the postulate survives, Locus will have a reusable owner-selection primitive
for service loops that want to collect only owners with new remote-free
activity.
