# Postulate 0156: Remote-Free Mixed Trace Repeated Summary

Date: 2026-07-03

## Claim

The remote-free mixed-trace benchmark should print repeated pre-benchmark
counter summaries, not only one sample, before its Criterion timing runs.

## Rationale

Experiment 0143 showed that queue capacity can remove producer-visible
`full_count` while increasing owner-side release wait. The benchmark currently
prints repeated summaries for the producer backpressure path, but the mixed
trace path only prints one pre-benchmark counter sample for each capacity.

That asymmetry is weak evidence for policy work. A single mixed-trace sample can
miss scheduling noise in the forced-drain path. Repeating the real trace before
Criterion timing makes it easier to distinguish stable release-latency behavior
from incidental host scheduling.

## Experiment

Extend `crates/locus-alloc/benches/remote_free_backpressure.rs` so each
mixed-trace capacity prints an eight-run summary line with:

- `full_count` min, max, and mean;
- forced drain min, max, and mean;
- drain round min, max, and mean;
- max pending min, max, and mean;
- max wait min, max, and mean;
- mean wait min, max, and mean.

Keep the workload unchanged: 256 real 4 KiB `Vec` allocations submitted as
eight bursts of 32 blocks.

## Falsification

The postulate is weakened if repeated summaries add noisy or misleading output,
hide the one-sample line used by prior notes, or require changing the mixed
trace workload itself.

## Expected Value

If the postulate survives, future remote-free policy changes can cite repeated
counter evidence for release latency and memory pressure instead of relying on a
single pre-benchmark sample.
