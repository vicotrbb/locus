# Postulate 0192: Remote-Free Guarded Runtime Sequence

Date: 2026-07-03

## Claim

Guarded remote-free retune decisions can drive `RemoteFreeOwnerRuntime`
install, confirm, rollback, and mutation-limit outcomes in one measured
sequence while preserving real owner-side allocation and release counters.

## Rationale

Experiments 0197 through 0199 added the typed applicator and the owner runtime
operations needed to apply, confirm, and roll back configs. The remaining gap
is integration: guard decisions should be observed as runtime operations, not
only as counters in the service planner or as isolated runtime operations.

## Experiment

Add a guarded runtime benchmark sequence that:

- feeds stable service summaries into `RemoteFreeServiceRetuneGuard`;
- translates apply decisions with `RemoteFreeServiceRetunePolicyApplicator`;
- installs applied configs through `RemoteFreeOwnerRuntime`;
- confirms the runtime after a clean validation window;
- rolls back the runtime after a failed validation window;
- treats mutation-limit decisions as runtime no-change outcomes;
- runs one real owner-runtime allocation window for every guard window.

The sequence should cover:

- one confirmed apply;
- one rolled-back apply;
- one mutation-limit decision after the apply budget is exhausted.

Every owner-runtime window should allocate real `Vec<u8>` blocks and release
them through owner-side runtime drains.

## Falsification

The postulate fails if runtime install, confirm, rollback, or mutation-limit
counters diverge from guard decisions, if runtime rollback state is stale after
confirmation or rollback, if final config differs from the expected rollback
state, or if real submitted, drained, released-byte, policy-drain, or wait
counters differ from the measured owner windows.

## Expected Value

If the postulate survives, Locus will have a single measured bridge from
service guard decisions to owner runtime operations before exploring live
multi-owner retune orchestration.
