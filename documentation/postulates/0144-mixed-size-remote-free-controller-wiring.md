# Postulate 0144: Mixed-Size Remote-Free Controller Wiring

Date: 2026-07-03

## Claim

`RemoteFreeDrainController` should preserve the mixed-size remote-free policy behavior from experiment 0147 when it replaces benchmark-local tracker and policy glue.

## Rationale

The controller now has request scratch and KV block domain evidence. The mixed-size trace remains important because it is the strongest current evidence that owner drain policy should consider retained bytes and release latency, not only queue capacity or producer backpressure.

This benchmark also exercises a nonblocking enqueue loop with forced drains on full-queue attempts, which is a different control path from the request and KV benchmarks.

## Experiment

Wire `remote_free_mixed_size_policy` through `RemoteFreeDrainController`.

Keep the existing workload:

- eight bursts of 32 blocks;
- mixed block sizes from 4 KiB to 32 KiB;
- `RemoteFreeQueue<TraceBlock>` with capacity 256 and batch 64;
- end-drain and max-wait-2 policies;
- nonblocking enqueue with forced drains on full queue attempts.

## Falsification

The postulate is weakened if controller wiring changes deterministic counters from experiment 0147, loses forced-drain accounting, hides the `TraceBlock` release behavior, or fails queue and tracker consistency checks.

## Expected Value

If the postulate survives, all three current remote-free policy benchmarks will share the same controller while preserving their domain-specific release logic.
