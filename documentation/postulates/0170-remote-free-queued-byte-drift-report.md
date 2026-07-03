# Postulate 0170: Remote-Free Queued-Byte Drift Report

Date: 2026-07-03

## Claim

A small typed drift report should compare a queued-byte drain config with live
remote-free queue and controller observations before Locus attempts adaptive
remote-free policy mutation.

## Rationale

Experiments 0175 through 0177 added `RemoteFreeQueuedByteDrainConfig` helpers
for grouped, uniform, and heterogeneous retained-work shapes. The remaining
open question is how production policy should react when `full_count`, pending
items, or retained queued bytes drift away from the configured target window.

An adaptive policy should not be introduced until the runtime can first report
the mismatch explicitly. The report should be pure, allocation-free, and usable
inside benchmark owner loops so we can validate it against real remote-free
allocation paths.

## Experiment

Add a focused `RemoteFreeQueuedByteDriftReport` that compares:

- configured target pending items;
- configured queued-byte budget;
- observed pending items;
- observed queued bytes;
- observed queue `full_count`.

Use saturating over-target counters so callers can decide whether to retune,
drain more often, increase queue capacity, or collect more evidence.

Wire the report into the mixed-size queued-byte benchmark case without changing
the benchmark workload, queue capacity, batch limit, drain closure, or policy
threshold.

## Falsification

The postulate is weakened if the report allocates, mutates policy, hides queue
backpressure, reports drift for the known-good mixed-size queued-byte config,
or fails to expose pending-item and retained-byte drift independently.

## Expected Value

If the postulate survives, future adaptive remote-free policy work can consume
a typed diagnostic instead of inferring drift from benchmark-local counters or
string output.
