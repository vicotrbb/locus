# Postulate 0216: Remote-Free Service Telemetry Timing Compare

Date: 2026-07-03

## Claim

The remote-free service telemetry JSON comparison tool can parse Criterion
timing intervals from the same saved benchmark outputs and emit a combined
report that refuses timing deltas when counter drift is present.

## Rationale

Experiment 0223 added a counter-stability gate for JSON sample rows. That
protects timing interpretation from changed allocation and service behavior,
but it still leaves timing extraction to manual review. The saved benchmark
output already contains the benchmark labels and Criterion `time:` intervals,
so the validation layer can join the stable sample comparison with normalized
timing deltas.

This keeps the rule explicit: counters first, timings only after counters are
stable.

## Experiment

Extend the remote-free service telemetry comparison tool so it:

- parses Criterion timing intervals for benchmarks present in JSON sample
  rows;
- normalizes timing values to picoseconds;
- rejects missing, duplicate, malformed, or unknown-unit timing intervals;
- compares baseline and candidate estimates by benchmark label;
- emits timing deltas only when sample counters are stable;
- emits a counter-drift status and no timing entries when sample counters
  drift.

## Falsification

The postulate fails if timing deltas are emitted when counters drift, if real
remote-free service telemetry benchmark output cannot be parsed, if duplicate
or malformed timing lines are accepted, if timing units are normalized
incorrectly, or if workspace format, clippy, tests, or benchmark compilation
fail.

## Expected Value

If the postulate survives, remote-free service telemetry benchmark reviews can
use one Rust validation command to prove counter stability and then inspect
normalized timing deltas from the same saved logs.
