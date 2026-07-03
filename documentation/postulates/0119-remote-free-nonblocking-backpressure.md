# Postulate 0119: Remote Free Nonblocking Backpressure

Date: 2026-07-03

## Statement

`RemoteFreeSink` should expose a nonblocking enqueue path that reports queue-full backpressure without stalling remote producers.

## Rationale

The current `RemoteFreeSink::enqueue` uses bounded-channel `send`, so a remote completion path can block when the owner is not draining quickly enough. That is acceptable for the original throughput benchmarks, but scheduler-facing runtime code needs to distinguish successful return, temporary queue congestion, and dropped-owner failure.

A nonblocking API should preserve the existing blocking path while adding explicit queue-full accounting. This lets later benchmarks measure batch policy tradeoffs and gives future schedulers a way to react to release congestion.

## Experiment

Add `RemoteFreeSink::try_enqueue` and associated try-enqueue error typing.

The change should:

- return the item on both full-queue and disconnected-owner failures;
- distinguish full-queue from disconnected-owner errors;
- count successful submissions, full-queue rejections, disconnected attempts, and owner-drained items;
- expose those counters through `RemoteFreeQueueStats`;
- keep the existing blocking `enqueue` behavior source compatible;
- add focused unit tests for full-queue backpressure, disconnected-owner accounting, and stats.

## Expected Result

The remote-free primitive should provide scheduler-ready congestion signals without changing existing benchmark behavior, and host plus Docker validation should keep passing.
