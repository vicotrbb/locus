# Postulate 0155: Remote-Free Controller Module

Date: 2026-07-03

## Claim

The remote-free policy, tracker, and controller code should live in a focused
submodule instead of sharing one large `remote_free.rs` file with the bounded
handoff queue.

## Rationale

`RemoteFreeQueue` is a bounded owner-drained handoff primitive. The policy,
tracker, and controller layer is owner-side runtime logic that interprets queue
state and scheduler turns. Keeping both families in one module makes future
runtime policy changes harder to review and increases the chance that queue
internals and policy accounting drift together.

A behavior-preserving extraction can keep the public `locus_alloc::*` API stable
while making the ownership boundary clearer:

- `remote_free.rs` owns queue and sink behavior;
- `remote_free/controller.rs` owns policy, tracker, controller, status, and
  controller accounting errors.

## Experiment

Move `RemoteFreeDrainPolicy`, `RemoteFreeDrainTracker`,
`RemoteFreeDrainController`, and related status, decision, reason, tracked
drain, and error types into a focused controller submodule.

The extraction should preserve:

- root re-exports from `locus_alloc`;
- queue behavior;
- controller unit tests;
- rustdoc owner-loop example;
- remote-free benchmark compilation.

## Falsification

The postulate is weakened if the extraction changes public API paths, breaks
domain benchmark compilation, hides release accounting from domain allocators,
or requires queue internals to own policy state.

## Expected Value

If the postulate survives, remote-free runtime policy code remains measured and
usable while the queue primitive stays focused on bounded handoff and draining.
