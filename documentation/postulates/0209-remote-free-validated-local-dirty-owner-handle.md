# Postulate 0209: Remote-Free Validated Local Dirty Owner Handle

Date: 2026-07-03

## Claim

The owner registry can issue a small validated local dirty-owner handle so
callers avoid manually passing owner limits while preserving the bounded
local dirty-buffer group behavior measured in Experiment 0216.

## Rationale

Experiment 0216 added `try_mark_dirty` and `try_local_marker`, which prevent
invalid or extremely sparse owner IDs from growing vector-indexed local dirty
buffer storage. Those methods still require every caller to pass the current
owner count. That is better than unchecked growth, but it spreads registry
knowledge into call sites.

The registry already owns the authoritative set of registered owners. A handle
validated by the registry can carry only the owner ID after validation, because
owner IDs are stable and the registry currently only grows. The local dirty
buffer group can then accept that handle for one-shot marking and marker
borrowing without repeating owner-limit plumbing at each call site.

## Experiment

Add a validated owner handle that:

- is created through the owner registry from a requested owner ID;
- rejects missing owner IDs without growing local buffer storage;
- exposes the validated owner ID for diagnostics;
- lets the local dirty-buffer group mark and borrow markers for accepted
  owners without a caller-provided owner limit;
- preserves local duplicate counters, flush counters, and tracked collection
  semantics;
- is compared against the bounded owner-limit path in the real allocation
  service-window sequence.

## Falsification

The postulate fails if missing owners can create handles, if rejected IDs grow
local buffer storage, if accepted handles change local duplicate or flush
counters, if service-window counters change, or if the validated-handle path is
measurably slower than the bounded path in the same allocation sequence.

## Expected Value

If the postulate survives, local dirty-buffer group callers can use a registry
validated handle as the normal production boundary and reserve direct
owner-limit calls for lower-level integration points.
