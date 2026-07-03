# Postulate 0140: KV Remote-Free Drain Tracker Benchmark

Date: 2026-07-03

## Claim

`RemoteFreeDrainTracker` should preserve the same latency and queued-byte policy signals when used with domain KV block handles, not only with `Vec` allocation traces.

## Rationale

Experiment 0147 moved remote-free pending-age and queued-byte accounting into a reusable tracker. That tracker is now used by the mixed-size `Vec` benchmark, but Locus is an inference allocator runtime. The tracker also needs evidence against domain handles from `KvBlockPool`, because KV-cache remote release is one of the central use cases.

## Experiment

Add a focused `kv_remote_free_policy` benchmark target.

Use:

- `KvBlockPool` with 256 blocks of 4 KiB;
- `RemoteFreeQueue<KvBlockHandle>` with capacity 256 and batch 64;
- a persistent remote completion thread;
- eight bursts of 32 KV handles;
- `RemoteFreeDrainTracker` to track 4 KiB per pending handle;
- `RemoteFreeDrainPolicy::new()` for end-drain;
- `RemoteFreeDrainPolicy::with_max_pending_age(2)` for max-wait-2.

The benchmark should print counters for submitted and drained handles, policy drains, drain rounds, max pending handles, peak queued bytes, released bytes, and logical wait.

## Falsification

The postulate is weakened if the tracker-backed max-wait-2 policy does not reduce max pending handles, peak queued bytes, and max wait versus end-drain, or if it introduces queue accounting drift against `RemoteFreeQueue` stats.

## Expected Value

If the postulate survives, Locus will have tracker evidence for both byte-buffer traces and domain KV handle remote-free paths.
