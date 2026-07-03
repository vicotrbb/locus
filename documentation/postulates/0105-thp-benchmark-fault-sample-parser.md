# Postulate 0105: THP Benchmark Fault Sample Parser

Date: 2026-07-03

## Statement

The mapped scratch THP benchmark fault sample lines should have typed parsers so benchmark output can be validated and consumed by future automation.

## Rationale

The THP write-touch benchmark now prints stable `fault_sample=` lines before Criterion timing. Those lines are useful only if downstream experiment logs and validation tools can reject malformed, duplicated, or incomplete samples.

Keeping the parser in `locus-alloc` matches the existing probe-output parser ownership. The parser should understand the three benchmark modes and preserve signed fault deltas.

## Experiment

Add parser support for:

- one `fault_sample=` line;
- multiline benchmark output containing exactly one `default`, `hugepage`, and `no_hugepage` sample;
- available samples with iterations and signed fault deltas;
- unavailable samples without numeric deltas.

## Expected Result

Focused unit tests should accept valid benchmark fault sample output and reject missing, duplicate, unknown, malformed, and internally inconsistent lines. Workspace validation should remain clean.
