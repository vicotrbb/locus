# Postulate 0168: Remote-Free Uniform Drain Config

Date: 2026-07-03

## Claim

`RemoteFreeQueuedByteDrainConfig` should support uniform retained item shapes
and should be used by the KV and request queued-byte benchmark policy cases.

## Rationale

Experiment 0175 added a grouped queued-byte drain config and proved it in the
owner-loop example. The KV and request queued-byte benchmarks still validate
their retained-byte budgets through `RemoteFreeQueuedByteBudget` only, while
queue capacity and drain batch sizing remain independent constants.

Both benchmarks have uniform retained item shapes:

- KV: 64 target pending blocks at 4096 bytes per block;
- request arenas: 8 target pending requests at 32 KiB per arena.

Those paths should use the same config validation as the owner-loop path:
queue capacity must hold the target window, and drain batch size must be able
to clear that target window.

## Experiment

Add `RemoteFreeQueuedByteDrainConfig::from_item_shape` and use it in the KV and
request queued-byte policy cases.

Keep benchmark names, queue capacities, batch limits, workloads, and release
closures unchanged.

## Falsification

The postulate is weakened if the config changes benchmark counters, weakens
the explicit owner release boundary, fails to reject invalid uniform sizing, or
makes the policy constructors harder to audit.

## Expected Value

If the postulate survives, queued-byte config validation will cover the grouped
owner-loop path and the two uniform real-allocation benchmark paths: KV block
handles and request-affine arena returns.
