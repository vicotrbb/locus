# Postulate 0169: Remote-Free Heterogeneous Drain Config

Date: 2026-07-03

## Claim

`RemoteFreeQueuedByteDrainConfig` should support heterogeneous retained item
shapes and should be used by the mixed-size queued-byte benchmark policy case.

## Rationale

Experiment 0176 moved uniform KV and request queued-byte benchmark policy cases
onto `RemoteFreeQueuedByteDrainConfig`. The mixed-size benchmark still derives
its queued-byte budget from a heterogeneous retained-size iterator and then
constructs a policy directly from the budget.

That benchmark has a concrete heterogeneous config shape:

- queue capacity: 256;
- drain batch limit: 64;
- target pending items: 64 inferred from the two-burst retained-size sequence;
- queued-byte budget: 655,360 bytes from the actual retained item sizes.

The config should infer the target pending window from the item-size iterator
so the item count and byte sum cannot drift.

## Experiment

Add `RemoteFreeQueuedByteDrainConfig::from_item_sizes`, counting retained items
and deriving the queued-byte budget from the same item-size iterator. Use it in
the mixed-size queued-byte policy case.

Keep benchmark names, queue capacity, batch limit, workload, and release
closure unchanged.

## Falsification

The postulate is weakened if the config changes mixed-size benchmark counters,
allows an empty heterogeneous sequence, misses item-count overflow, weakens
byte-budget validation, or hides owner-side release behavior.

## Expected Value

If the postulate survives, queued-byte config validation covers all currently
measured retained-work shapes: grouped owner-loop, uniform KV and request
benchmarks, and heterogeneous mixed-size allocation traces.
