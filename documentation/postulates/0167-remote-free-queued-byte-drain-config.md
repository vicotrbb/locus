# Postulate 0167: Remote-Free Queued-Byte Drain Config

Date: 2026-07-03

## Claim

A small queued-byte drain configuration type should validate queue capacity,
drain batch size, target pending item window, and retained-byte budget together.

## Rationale

The budget-selection note says a queued-byte threshold should be selected with
queue capacity and drain batch size in mind. The measured queued-byte paths all
use the same invariant:

- queue capacity can hold at least the target pending item window;
- drain batch size can clear that target window at a policy drain point;
- retained-byte budget is derived from the target pending item shape;
- owner release logic remains explicit in the queue drain closure.

Today those relationships live in examples, benchmarks, and prose. A small
configuration type can centralize the validation without making remote-free
queues own allocator-specific release behavior.

## Experiment

Add a `RemoteFreeQueuedByteDrainConfig` API that:

- validates non-zero queue capacity and drain batch limit;
- validates a non-zero target pending item window;
- rejects queue capacity or drain batch limits below the target window;
- derives grouped retained-byte budgets through
  `RemoteFreeQueuedByteBudget`;
- exposes queue capacity, drain batch limit, target pending items, budget, and
  queued-byte drain policy;
- can build a `RemoteFreeQueue<T>` with the validated queue parameters.

Wire the queued-byte owner-loop example through the config while preserving the
existing counters and explicit release closure.

## Falsification

The postulate is weakened if the config hides owner-side release behavior,
duplicates benchmark-only logic, lets invalid queue or batch sizing through, or
changes the owner-loop example counters.

## Expected Value

If the postulate survives, future runtime configuration work can build on a
tested API that ties retained-byte policy to the queue and drain sizing
required by the measured remote-free behavior.
