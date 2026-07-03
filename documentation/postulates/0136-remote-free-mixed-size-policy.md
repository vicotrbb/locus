# Postulate 0136: Remote-Free Mixed-Size Policy

Date: 2026-07-03

## Claim

A latency-bounded owner drain policy can reduce peak queued remote-free bytes for mixed allocation sizes without increasing producer-visible backpressure, even when the queue capacity is large enough to hold the whole trace.

## Rationale

Experiment 0143 showed that capacity 256 removed `full_count` for a 256 item trace, but it increased logical release wait and peak pending item count. Real inference memory pressure is driven by bytes, not only item count. A queue containing mixed activation, staging, or request-local buffers can retain much more memory than a fixed 4 KiB item benchmark suggests.

The next benchmark should keep capacity large, then compare policy behavior:

- fixed large queue with end-of-trace draining;
- same capacity with a latency bound that drains before pending work reaches the full queue.

## Experiment

Add `remote_free_mixed_size_policy`, a Criterion benchmark that submits 256 real `Vec` allocations as eight bursts of 32 blocks. Block sizes cycle through a mixed-size pattern from 4 KiB to 32 KiB.

Compare:

- capacity 256, batch 64, fixed end drain;
- capacity 256, batch 64, latency-bound drain every two bursts.

Record:

- submitted and drained counts;
- `full_count`;
- forced drains caused by a full queue;
- policy drains caused by the latency bound;
- drain rounds;
- maximum pending item count;
- peak queued bytes;
- released bytes;
- maximum and mean logical wait in bursts.

## Falsification

The postulate is weakened if the latency-bounded policy increases `full_count`, has worse timing without reducing peak queued bytes, or fails to reduce maximum and mean logical wait.

## Expected Value

If the postulate survives, Locus should avoid defaulting to large remote-free capacity alone. A future scheduler-facing policy should include queued-byte or release-latency triggers in addition to queue-full backpressure.
