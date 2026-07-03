# Postulate 0163: Remote-Free Queued-Byte Budget Helper

Date: 2026-07-03

## Claim

A small typed helper for deriving remote-free queued-byte budgets will reduce
duplicated sizing arithmetic without weakening the explicit owner-drained
runtime boundary.

## Rationale

Experiments 0166, 0168, and 0169 showed that queued-byte policy thresholds can
match the current max-wait-2 counter behavior across mixed-size allocation
traces, KV block handles, and request-affine arena returns.

The current owner-loop example still computes:

- request concurrency times blocks per request;
- retained block count times representative block bytes;
- non-zero validation for the resulting policy threshold.

That calculation is small, but it is exactly the sort of code that will spread
across domain allocators as queued-byte policy becomes a runtime configuration
candidate. The helper should centralize checked multiplication and zero
validation while keeping policy choice explicit at call sites.

## Experiment

Add a `RemoteFreeQueuedByteBudget` helper to `locus-alloc` that:

- stores a non-zero queued-byte budget;
- derives budgets from item count and bytes per item;
- derives budgets from grouped item shapes such as requests, blocks per
  request, and bytes per block;
- reports zero and overflow failures with a public error type;
- exposes the underlying `NonZeroU64` for `RemoteFreeDrainPolicy`;
- can build a queued-byte-only `RemoteFreeDrainPolicy` for simple call sites.

Wire the queued-byte owner-loop example through the helper and keep the release
logic in the `RemoteFreeQueue::drain_batch` closure.

## Falsification

The postulate is weakened if the helper hides domain release logic, makes
policy composition harder, duplicates controller responsibilities, misses
overflow cases, or changes the owner-loop example counters.

## Expected Value

If the postulate survives, retained-byte policy configuration becomes easier to
reuse in later KV-cache, request-affine, and GPU-near staging experiments
without repeating hand-rolled arithmetic at each call site.
