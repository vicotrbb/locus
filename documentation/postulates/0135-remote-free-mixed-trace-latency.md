# Postulate 0135: Remote-Free Mixed Trace Latency

Date: 2026-07-03

## Claim

Increasing remote-free queue capacity can reduce nonblocking enqueue backpressure while hiding longer owner-side release latency.

## Rationale

Experiment 0142 showed that larger capacity queues can produce zero repeated pre-sample `full_count` observations and slightly better short Criterion timing. That is useful, but it is not enough to choose a runtime default. A larger queue may let remote completions keep submitting work while the owning worker delays actual release work for more scheduling turns.

For an inference allocator, that matters because a release path that looks fast at the producer can still retain memory longer on the owner. The next benchmark should therefore test both:

- `full_count`, to measure producer-visible backpressure;
- logical release wait, to measure how many burst turns queued allocations remain pending before owner release.

## Experiment

Add a mixed remote-free trace benchmark that submits 256 real 4 KiB `Vec` allocations as eight bursts of 32 blocks. The owner only drains at the end of the trace unless the bounded queue fills, which forces an early owner drain.

Compare:

- capacity 64, batch 64;
- capacity 128, batch 64;
- capacity 256, batch 64.

The benchmark should print a pre-benchmark sample with:

- submitted and drained counts;
- `full_count`;
- forced drain count;
- drain round count;
- maximum pending count;
- maximum and mean wait in logical bursts.

## Falsification

The postulate is weakened if larger capacity reduces `full_count` without increasing maximum or mean logical wait. It is also weakened if timing becomes worse enough that capacity no longer looks useful even before considering release latency.

## Expected Value

If the postulate survives, capacity should not become a default tuning knob on throughput evidence alone. The runtime will need a policy that considers both remote enqueue backpressure and owner-side release latency.
