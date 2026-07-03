# Postulate 0103: Process Fault Count Observability

Date: 2026-07-03

## Statement

Locus should parse current-process minor and major fault counters so allocation and first-touch benchmarks can record page-fault evidence.

## Rationale

The benchmark plan now includes mapped scratch THP first-touch cost, but timing alone does not explain whether a run changed page fault behavior. Linux exposes per-process minor and major fault counters in `/proc/<pid>/stat`.

Reusable parsing in `locus-observe` keeps future probes and benchmarks from duplicating fragile `/proc` parsing. The parser must handle command names inside parentheses before reading the numeric fields.

## Experiment

Add a `ProcessFaultCounts` type plus parser and reader helpers for `/proc/self/stat`.

The parser should extract:

- minor faults;
- child minor faults;
- major faults;
- child major faults.

## Expected Result

The parser should pass fixture tests, the reader should work in Docker when `/proc/self/stat` is available, and the workspace validation gates should remain clean.
