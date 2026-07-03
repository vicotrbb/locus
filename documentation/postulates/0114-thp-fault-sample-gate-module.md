# Postulate 0114: THP Fault Sample Gate Module

Date: 2026-07-03

## Statement

The mapped scratch THP fault sample validation gate should live in a dedicated `locus-validate` module instead of the broad validation root file.

## Rationale

The root validation file currently owns pinned scratch gates, mapped scratch THP gates, THP fault sample gate evaluation, and parser tests. The fault sample comparison and report parsers already live in focused modules, but the base fault sample gate still sits in `lib.rs`.

Moving the gate types, parser errors, parser functions, evaluation function, and focused tests into `thp_fault_sample_gate.rs` should make the API boundary easier to review while preserving existing public re-exports.

## Experiment

Extract the mapped scratch THP fault sample validation gate into `crates/locus-validate/src/thp_fault_sample_gate.rs`.

The module should own:

- gate status, reason, gate, and verdict types;
- line, output, and evaluation parse errors;
- display implementations;
- gate evaluation from parsed benchmark fault samples;
- line and multiline output parsers;
- focused tests for ready, unavailable, malformed, duplicate, and inconsistent outputs.

## Expected Result

The root validation file should shrink, the public API should remain source compatible, and `cargo test -p locus-validate` should pass with unchanged behavior.
