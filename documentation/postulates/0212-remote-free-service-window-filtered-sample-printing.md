# Postulate 0212: Remote-Free Service-Window Filtered Sample Printing

Date: 2026-07-03

## Claim

The service-window benchmark harness can make its sample printing aware of the
Criterion benchmark filter while preserving unfiltered sample output and
benchmark execution.

## Rationale

Experiment 0219 showed that descriptor registration preserved benchmark
identity, but a focused Criterion run still printed service-window samples for
benchmarks that the filter would not execute. That makes real benchmark output
harder to inspect and increases the chance of misreading a noisy timing block.

The service-window harness already knows each benchmark name next to each
sample label. It can use the same command-line filter token that Criterion
receives to decide whether a sample and sample summary should be printed.
This should be a presentation-only change; benchmark registration and
assertions must remain unchanged.

## Experiment

Gate service-window sample and sample-summary printing with a small filter
helper that:

- prints all service-window samples when no Criterion filter is present;
- prints a service-window sample when its benchmark name or sample label
  matches a filter token;
- suppresses unrelated service-window samples for exact focused filters;
- keeps the existing Criterion benchmark names and descriptors unchanged;
- still runs the focused benchmark and preserves allocation counters.

## Falsification

The postulate fails if an unfiltered run suppresses service-window samples, if
an exact focused service-window filter prints unrelated service-window sample
labels, if a matching sample is suppressed, if benchmark names change, or if
the filtered benchmark counters change.

## Expected Value

If the postulate survives, service-window benchmark output becomes easier to
read during focused experiments without changing the measured allocation path.
