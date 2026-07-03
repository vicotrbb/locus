# Postulate 0179: Request Remote-Free Retune Action

Date: 2026-07-03

## Claim

The request-affine arena remote-free benchmark should report
`RemoteFreeQueuedByteRetuneAction` from a queued-byte drift report while
preserving real request arena close behavior and measured queued-byte counters.

## Rationale

Experiment 0186 validated `retune_action` on real KV block handles. Request
arenas are the other current domain owner loop with explicit remote-free
release behavior. The request benchmark closes real request-scoped arenas
inside the owner drain closure and derives the queued-byte target from arena
capacity, so it should provide the next domain-specific evidence point.

## Experiment

Wire `RemoteFreeQueuedByteDriftReport` into `request_remote_free_policy` and
assert:

- end-drain reports retained-window drift and `retune_action=drain_earlier`;
- max-wait-2 reports no drift and `retune_action=keep_config`;
- max-queued-256KiB reports no drift and `retune_action=keep_config`.

Keep the workload unchanged:

- 16 real request arenas;
- four bursts of four requests;
- 32 KiB arena capacity;
- queue capacity 16;
- drain batch limit 8;
- owner release stays inside `RequestScratchPool::close_request`.

## Falsification

The postulate is weakened if retune-action logging changes request counters,
requires moving `RequestScratchPool::close_request` out of the owner drain
closure, or fails to distinguish end-drain retained-window drift from the
queued-byte policy case.

## Expected Value

If the postulate survives, `retune_action` has evidence across generic traces,
runtime-facing examples, KV handles, and request-affine arenas.
