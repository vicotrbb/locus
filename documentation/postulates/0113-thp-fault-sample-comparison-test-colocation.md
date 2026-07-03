# Postulate 0113: THP Fault Sample Comparison Test Colocation

Date: 2026-07-03

## Statement

THP fault sample comparison parser tests should live beside the comparison parser implementation instead of in the broad `locus-validate` root test module.

## Rationale

The comparison output and parser were moved into `thp_fault_sample_comparison.rs` to avoid growing the large validation root file. The parser tests still live in `lib.rs`, which weakens that module boundary and makes future comparison parser changes harder to review in isolation.

Colocating those tests keeps parser behavior, edge cases, and private schema assumptions near the parser implementation while preserving root-level public re-exports.

## Experiment

Move the comparison line and output parser tests into `crates/locus-validate/src/thp_fault_sample_comparison.rs`.

The moved tests should still cover:

- valid available and unavailable comparison lines;
- invalid status, reason, numeric, boolean, duplicate, missing, unknown, and inconsistent fields;
- valid multiline comparison extraction;
- missing, duplicate, and malformed multiline comparison output.

## Expected Result

Focused validation should keep the same comparison parser coverage while `lib.rs` gets smaller and no public API behavior changes.
