# Postulate 0215: Remote-Free Service Telemetry JSON Compare

Date: 2026-07-03

## Claim

The optional remote-free service telemetry JSON sample rows can be consumed by
a small Rust validation tool that compares two benchmark outputs and reports
counter drift before timing deltas are trusted.

## Rationale

Experiment 0222 made every remote-free service telemetry sample row available
as typed JSON when `LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON` is enabled.
Those rows are useful only if follow-up experiments can compare two real runs
without writing one-off shell or Python filters. A reusable parser and
comparison report in `locus-validate` matches the repository pattern for saved
benchmark evidence parsers and keeps the comparison logic testable.

## Experiment

Add a focused `locus-validate` module and example command that:

- reads two saved benchmark outputs;
- extracts remote-free service telemetry JSON sample rows;
- keys rows by `(benchmark, sample)`;
- rejects malformed JSON rows, duplicate sample keys, and missing sample sets;
- compares all parsed `fields` values exactly;
- prints a compact stable or drift report and explicit drift lines;
- passes focused unit tests and real benchmark-output checks.

## Falsification

The postulate fails if the tool accepts malformed JSON, ignores duplicate
sample rows, misses changed counters, reports stable when a sample is missing,
cannot parse the JSON rows emitted by real benchmark runs, or makes the
workspace fail format, clippy, tests, or benchmark compilation.

## Expected Value

If the postulate survives, future remote-free service telemetry timing
experiments can first prove that allocation and service counters stayed stable
across runs. Timing deltas can then be reviewed only after counter drift has
been ruled out by reusable Rust tooling.
