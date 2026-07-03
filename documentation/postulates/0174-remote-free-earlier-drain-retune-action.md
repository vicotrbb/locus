# Postulate 0174: Remote-Free Earlier-Drain Retune Action

Date: 2026-07-03

## Claim

When increasing queue capacity causes `review_multiple_signals`, adding an
earlier queued-byte drain trigger should remove backpressure without increasing
the retained-byte window or owner-side release wait.

## Rationale

Experiment 0181 showed that capacity alone is not a complete adaptive action.
Capacity 256 removed `full_count`, but allowed the queue to retain 256 pending
items, 1,048,576 bytes, max wait 8 bursts, and mean wait 4.500 bursts against
a 64-item target window.

The next candidate is to keep larger capacity for producer slack while using
the queued-byte config as an owner-side drain trigger.

## Experiment

Extend the capacity retune benchmark with larger-capacity policy cases:

- capacity 128 with queued-byte policy from the 64-item config;
- capacity 256 with queued-byte policy from the 64-item config.

Assert that both policy cases keep `full_count=0`, max pending 64, max queued
bytes 262,144, max wait 2 bursts, mean wait 1.500 bursts, and retune hint
`keep_config`.

## Falsification

The postulate is weakened if earlier drains fail to remove backpressure, if
they still exceed the configured target window, if release wait grows beyond
the capacity-64 baseline, or if the benchmark relies on synthetic counters
instead of real allocation and queue activity.

## Expected Value

If the postulate survives, the adaptive path becomes clearer: capacity handles
producer slack, while queued-byte drains preserve the retained-memory and
latency window.
