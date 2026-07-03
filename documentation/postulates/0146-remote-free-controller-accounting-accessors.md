# Postulate 0146: Remote-Free Controller Accounting Accessors

Date: 2026-07-03

## Claim

`RemoteFreeDrainController` should expose direct owner-side accounting accessors so runtime code and benchmarks do not need to reach through `controller.tracker()` for common pending count, queued byte, and empty-state checks.

## Rationale

Experiments 0150 through 0153 moved remote-free policy wiring into `RemoteFreeDrainController` and added a runtime-facing owner-loop example. The benchmark call sites still use `controller.tracker().queued_bytes()`, `controller.tracker().pending_count()`, and `controller.tracker().is_empty()`.

The tracker remains useful for detailed inspection, but common controller users should not need to depend on the tracker shape for normal policy-loop accounting.

## Experiment

Add direct controller accessors:

- `pending_count`;
- `queued_bytes`;
- `is_empty`.

Update the controller tests, rustdoc example, and policy benchmarks to use those methods while keeping `tracker()` available for explicit low-level inspection.

## Falsification

The postulate is weakened if the accessors drift from tracker accounting, complicate the API, or change benchmark behavior.

## Expected Value

If the postulate survives, runtime owner-loop code will read more clearly while preserving the explicit release-closure design.
