# Postulate 0110: THP Fault Sample Comparison Parser

Date: 2026-07-03

## Statement

The mapped scratch THP fault sample comparison output should have typed parsers so report aggregation can consume the stable comparison line without ad hoc string splitting.

## Rationale

The validation command now prints `mapped_scratch_thp_fault_sample_comparison=` lines after the fault sample gate verdict. Those lines are useful for saved benchmark reports only if downstream tooling can reject malformed fields, duplicate fields, inconsistent status and reason pairs, and incomplete available comparisons.

This parser should preserve the existing conservative interpretation. It parses a comparison line as benchmark evidence, not as proof of transparent huge page adoption.

## Experiment

Add parser support in `locus-validate` for:

- one available comparison line with all required delta fields;
- one unavailable comparison line with no delta fields;
- multiline output containing exactly one comparison line;
- duplicate, missing, unknown, malformed, and inconsistent comparison line rejection.

## Expected Result

Focused tests should accept valid available and unavailable comparison lines, reject malformed variants, reject duplicate output lines, and keep workspace validation clean.
