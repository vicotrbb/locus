# Postulate 0122: Remote Free Repeated Backpressure Samples

Date: 2026-07-03

## Statement

The remote-free backpressure benchmark should print repeated pre-benchmark counter samples instead of a single `full_count` observation.

## Rationale

Experiments 0128 and 0129 showed that one-run `full_count` observations are schedule-sensitive. A single sample is still useful as a sanity check, but it is too weak to guide scheduler policy. Repeating the sample several times and reporting min, max, and mean full-queue retries should make the evidence more honest without changing the Criterion timing workload.

## Experiment

Update `remote_free_backpressure` so each benchmark case prints:

- the existing one-run sample line;
- a repeated sample summary over eight runs;
- min, max, and mean `full_count`;
- min, max, and mean `pending_count`.

The Criterion benchmark bodies should stay unchanged so timing comparisons remain comparable to experiment 0129.

## Expected Result

The benchmark should compile under all-target checks and produce both timing data and a more stable counter summary for all four capacity and batch cases.
