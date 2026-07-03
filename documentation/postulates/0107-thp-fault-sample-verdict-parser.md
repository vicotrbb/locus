# Postulate 0107: THP Fault Sample Verdict Parser

Date: 2026-07-03

## Statement

The mapped scratch THP fault sample validation gate should provide typed parsers for its own stable verdict line.

## Rationale

The gate now emits `mapped_scratch_thp_fault_sample_validation_gate=<status> reason=<reason>`, but downstream report aggregation cannot consume that verdict with the same typed guarantees as the other validation gates.

Adding line and output parsers keeps the gate surface symmetrical with pinned scratch, near-GPU pinned scratch, mapped scratch THP, and placement validation gates.

## Experiment

Add parser support for:

- one fault sample validation gate verdict line;
- multiline output containing exactly one fault sample validation gate line;
- ready and unavailable verdicts;
- duplicate, missing, unknown, extra-token, and inconsistent verdict rejection.

## Expected Result

Focused tests should accept valid ready and unavailable verdicts, reject malformed verdict lines, reject duplicate output verdicts, and keep workspace validation clean.
