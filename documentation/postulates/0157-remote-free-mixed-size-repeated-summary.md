# Postulate 0157: Remote-Free Mixed-Size Repeated Summary

Date: 2026-07-03

## Claim

The remote-free mixed-size policy benchmark should print repeated counter
summaries for retained bytes and release latency before Criterion timing.

## Rationale

The mixed-size benchmark is the strongest current evidence that owner-side
remote-free policy should consider retained bytes and release latency, not only
queue capacity. It currently prints one pre-benchmark sample for the end-drain
and max-wait-2 policies.

A single sample is useful for deterministic trace shape, but repeated summaries
are better evidence for future policy work. They make it harder to accidentally
treat an incidental run as a stable policy signal, and they align the
mixed-size policy benchmark with the repeated summary pattern added to the
mixed-trace benchmark.

## Experiment

Extend `crates/locus-alloc/benches/remote_free_mixed_size_policy.rs` so each
policy case prints an eight-run summary line with min, max, and mean for:

- `full_count`;
- forced drains;
- policy drains;
- drain rounds;
- max pending count;
- max queued bytes;
- max wait bursts;
- mean wait bursts.

Keep the workload unchanged: 256 real `Vec` allocations over eight bursts with
the existing mixed-size pattern.

## Falsification

The postulate is weakened if repeated summaries change benchmark workload
behavior, hide the existing one-run sample line, or produce unstable retained
byte and wait summaries for the deterministic single-threaded trace.

## Expected Value

If the postulate survives, queued-byte policy experiments can cite repeated
counter evidence for both memory retention and release latency before changing
runtime defaults.
