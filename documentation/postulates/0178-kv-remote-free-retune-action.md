# Postulate 0178: KV Remote-Free Retune Action

Date: 2026-07-03

## Claim

The KV remote-free policy benchmark should report
`RemoteFreeQueuedByteRetuneAction` from the same drift report used by generic
remote-free traces, while preserving real `KvBlockHandle` release behavior and
the measured queued-byte counters.

## Rationale

Experiment 0185 showed how a runtime-facing owner loop can log
`retune_action=keep_config` from a queued-byte drift report. KV block release is
a central inference-memory path, so the same action logging should be validated
against real KV handles, not only representative `Vec` allocations.

The existing KV benchmark already compares end-drain, max-wait-2, and
queued-byte policy cases. It can use the 64-block, 262,144-byte queued-byte
config as the drift target for all three cases.

## Experiment

Wire `RemoteFreeQueuedByteDriftReport` into `kv_remote_free_policy` and assert:

- end-drain reports retained-window drift and `retune_action=drain_earlier`;
- max-wait-2 reports no drift and `retune_action=keep_config`;
- max-queued-256KiB reports no drift and `retune_action=keep_config`.

Keep the workload unchanged:

- 256 real KV block handles;
- eight bursts of 32 handles;
- block size 4096 bytes;
- queue capacity 256;
- drain batch limit 64;
- owner release stays inside `KvBlockPool::free`.

## Falsification

The postulate is weakened if retune-action logging changes KV counters,
requires moving `KvBlockPool::free` out of the owner drain closure, or fails to
distinguish end-drain retained-window drift from the queued-byte policy case.

## Expected Value

If the postulate survives, `retune_action` has evidence on a real KV-cache
handle path and can guide domain owner-loop logging beyond generic allocation
traces.
