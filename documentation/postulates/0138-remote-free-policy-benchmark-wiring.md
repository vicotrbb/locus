# Postulate 0138: Remote-Free Policy Benchmark Wiring

Date: 2026-07-03

## Claim

The mixed-size remote-free benchmark should use `RemoteFreeDrainPolicy` directly instead of a local benchmark-only policy enum.

## Rationale

Experiment 0145 added a pure owner-drain policy model. Experiment 0144 still measures a benchmark-local `DrainPolicy`. That leaves a gap: the best queued-byte policy result is documented, but the benchmark does not yet prove that the reusable production policy model can express the same behavior.

The benchmark should keep its real allocation trace and queue counters, while replacing local policy logic with:

- `RemoteFreeDrainObservation`;
- `RemoteFreeDrainPolicy`;
- `RemoteFreeDrainDecision`.

Because `RemoteFreeQueue` should not expose item internals, the benchmark can maintain external pending-age metadata for observations.

## Experiment

Update `remote_free_mixed_size_policy` so:

- the end-drain case uses `RemoteFreeDrainPolicy::new()`;
- the max-wait-2 case uses `RemoteFreeDrainPolicy::with_max_pending_age`;
- the benchmark builds observations from pending count, queued bytes, and oldest pending age;
- the existing counters and timings remain comparable to experiment 0144.

## Falsification

The postulate is weakened if using the production policy changes the observed counters, makes the benchmark materially more complex, or cannot reproduce the max-wait-2 policy result.

## Expected Value

If the postulate survives, future remote-free benchmarks and runtime loops can use the same policy type instead of duplicating threshold logic.
