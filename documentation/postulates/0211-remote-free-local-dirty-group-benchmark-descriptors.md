# Postulate 0211: Remote-Free Local Dirty Group Benchmark Descriptors

Date: 2026-07-03

## Claim

The local dirty-buffer group service-window benchmarks can be registered from
a descriptor table while preserving the existing Criterion benchmark names,
sample output labels, allocation counters, and validation behavior.

## Rationale

Experiment 0218 centralized the local dirty-buffer group collection
assertions, but the Criterion registration surface still has one wrapper per
group mode. Each wrapper repeats the same shape: print one sample, print one
sample summary, then run `run_runtime_service_window_sequence` for a mode.

That wrapper repetition is low risk today, but it makes the next group mode
require edits in multiple places. A descriptor table should make the mode list
the single source of truth while keeping benchmark output stable.

## Experiment

Replace the four local dirty-buffer group Criterion wrappers with one
registration function backed by descriptors containing:

- runner mode;
- sample label;
- sample summary label;
- Criterion benchmark name.

The refactor must keep the existing benchmark names and sample labels, compile
the benchmark target, run the focused local dirty group benchmark filter, and
preserve the same service-window counters.

## Falsification

The postulate fails if a benchmark name changes, if a sample label changes, if
any local dirty group service-window counter changes, if the focused benchmark
filter no longer runs all four group cases, or if clippy rejects the table
driven registration code.

## Expected Value

If the postulate survives, future local dirty-buffer group benchmark modes can
be added by adding one descriptor instead of another near-identical wrapper
function and Criterion entrypoint call.
