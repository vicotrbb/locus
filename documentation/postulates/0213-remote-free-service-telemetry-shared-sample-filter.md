# Postulate 0213: Remote-Free Service Telemetry Shared Sample Filter

Date: 2026-07-03

## Claim

The remote-free service telemetry benchmark target can share one filter-aware
sample printing helper across harness modules so focused Criterion runs
suppress unrelated sample blocks target-wide while preserving benchmark
registration and allocation counters.

## Rationale

Experiment 0220 made `runtime_service_window_harness.rs` filter-aware, but the
other remote-free service telemetry harnesses still print samples before
Criterion filters benchmark execution. Exact focused runs are therefore still
noisy at the target level, which makes real timing output harder to audit.

A small shared helper can parse Criterion filter tokens once per benchmark
process and compare them with the sample label and Criterion benchmark label
provided by each harness. Keeping the helper at the benchmark-target layer
avoids pulling Criterion or argument parsing into production crates.

## Experiment

Add a shared benchmark helper that:

- prints every sample when no Criterion filter token is present;
- prints a sample when its sample label or benchmark label matches a filter
  token;
- suppresses unrelated samples across all remote-free service telemetry
  harness modules for exact focused filters;
- preserves existing Criterion benchmark names;
- preserves focused benchmark allocation counters and validation behavior.

## Falsification

The postulate fails if an exact focused benchmark filter still prints unrelated
sample labels from any remote-free service telemetry harness, if unfiltered
registration suppresses samples, if benchmark names change, if benchmark
compilation fails, or if measured counters change.

## Expected Value

If the postulate survives, benchmark output becomes easier to audit during
focused experiments, and future harness modules can reuse one small filtering
surface instead of adding local argument parsing.
