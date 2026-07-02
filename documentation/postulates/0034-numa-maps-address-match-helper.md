# Postulate 0034: Numa Maps Address Match Helper

Date: 2026-07-02

## Statement

`locus-observe` should expose one helper that returns the best `numa_maps` entry match for an address and records whether it was an exact start or containing-range match.

## Rationale

Validation probes currently need both exact-start and containing-address lookup. Keeping that logic in each probe risks inconsistent output and match precedence. A typed helper gives probes one source of truth and makes the match kind machine-readable.

## Experiment

Add:

- a `NumaMapsAddressMatchKind` enum;
- a `NumaMapsAddressMatch` result type;
- a `numa_maps_entry_for_address` helper that prefers exact-start matches;
- fixture tests for exact, containing, and missing addresses.

## Expected Result

The helper should pass workspace tests and clippy. Exact mapping starts should be preferred over containing-range matches.
