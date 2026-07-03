# Postulate 0145: Remote-Free Owner Loop Example

Date: 2026-07-03

## Claim

`RemoteFreeDrainController` needs a runtime-facing owner-loop example that is compile-checked and tested with real allocated buffers, not only benchmark-local call sites.

## Rationale

Experiments 0150, 0151, and 0152 proved that the controller preserves request scratch, KV block, and mixed-size policy counters. Runtime users still need a clear pattern for combining:

- `RemoteFreeQueue`;
- remote submit accounting;
- policy decisions;
- explicit domain release logic;
- FIFO drain accounting.

The controller should not hide allocator-specific release calls, but docs should show where those calls belong.

## Experiment

Add rustdoc guidance to `RemoteFreeDrainController` with a compile-checked owner-loop example.

Add a focused unit test that mirrors the example with real `Vec` buffers, `RemoteFreeQueue`, `RemoteFreeDrainPolicy`, and `RemoteFreeDrainController`.

## Falsification

The postulate is weakened if the example needs hidden internals, does not compile under doc tests, uses fake release accounting, or cannot exercise real allocated buffers.

## Expected Value

If the postulate survives, runtime users will have a small verified template for using the controller while keeping release logic explicit in domain allocator code.
