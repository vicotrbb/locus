# Postulate 0214: Remote-Free Service Telemetry JSON Sample Lines

Date: 2026-07-03

## Claim

The remote-free service telemetry benchmark target can emit optional
machine-readable JSON sample lines from the same sample rows used for human
benchmark review, while preserving default text output, Criterion benchmark
registration, and allocation counters.

## Rationale

Experiment 0221 made focused sample output filter-clean across the whole
benchmark target. The remaining weakness is that future experiments still
compare counters by grepping text lines. A benchmark-scoped output helper can
keep the existing human-readable `key=value` rows as the source of truth and,
only when explicitly requested, emit a JSON object with the benchmark label,
sample label, and parsed fields.

Keeping this helper inside the benchmark target avoids production dependency
growth and keeps the output path outside measured iteration loops.

## Experiment

Add an optional JSON output helper that:

- leaves default text output unchanged;
- emits one additional JSON object per printed sample row when enabled by an
  environment variable;
- preserves the existing sample filter behavior;
- parses numeric and boolean field values into JSON scalars when possible;
- escapes JSON strings without adding a benchmark dependency;
- preserves focused benchmark counters and timings.

## Falsification

The postulate fails if default benchmark output changes, if focused filters
print unrelated JSON rows, if JSON rows are malformed, if important counters
cannot be extracted from JSON, if benchmark registration changes, if the bench
target fails to compile, or if measured allocation counters change.

## Expected Value

If the postulate survives, future remote-free service telemetry experiments can
compare counter fields from JSON lines instead of relying on ad hoc text
filters, while still keeping the current text logs useful for humans.
