# Postulate 0143: KV Remote-Free Controller Wiring

Date: 2026-07-03

## Claim

`RemoteFreeDrainController` should preserve the measured KV remote-free policy behavior from experiment 0148 when it replaces benchmark-local tracker and policy glue.

## Rationale

Experiment 0150 proved the controller on the request scratch arena return path. KV block remote-free is the other high-value domain path because it returns allocator-owned block handles through `RemoteFreeQueue<KvBlockHandle>` and releases them back to `KvBlockPool`.

The controller should keep domain release logic explicit while sharing:

- submit accounting;
- queue and tracker pending-count consistency checks;
- policy decisions;
- FIFO drain accounting.

## Experiment

Wire the `kv_remote_free_policy` benchmark through `RemoteFreeDrainController`.

Keep the existing workload:

- `KvBlockPool` with 256 blocks of 4 KiB;
- `RemoteFreeQueue<KvBlockHandle>` with capacity 256 and batch 64;
- eight bursts of 32 handles;
- a persistent remote completion thread;
- end-drain and max-wait-2 policies.

## Falsification

The postulate is weakened if controller wiring changes the deterministic counters from experiment 0148, hides `KvBlockPool::free`, or fails queue and tracker consistency checks.

## Expected Value

If the postulate survives, the reusable controller will have domain evidence across request scratch arenas and KV block handles.
