# Postulate 0115: Mapped Scratch THP Gate Module

Date: 2026-07-03

## Statement

The mapped scratch transparent huge page validation gate should live in a focused `locus-validate` module instead of the broad validation root file.

## Rationale

The THP fault sample gate, comparison parser, and report parser now live in dedicated modules. The earlier mapped scratch THP validation gate still sits in `lib.rs`, along with pinned scratch and near-GPU validation gates. Keeping this gate in the root file weakens the module boundary for THP validation and makes future THP-specific changes harder to review.

Moving the gate types, parser errors, evaluation function, gate-line parsers, and focused tests into `mapped_scratch_thp_gate.rs` should shrink `lib.rs` while preserving the public root re-exports.

## Experiment

Extract the mapped scratch THP validation gate into `crates/locus-validate/src/mapped_scratch_thp_gate.rs`.

The module should own:

- gate status, reason, gate, and verdict types;
- line, output, and evaluation parse errors;
- display implementations;
- probe evaluation from parsed mapped scratch THP output;
- line and multiline gate parsers;
- focused tests for ready, unavailable, not-ready, parser-error, duplicate, malformed, and inconsistent cases.

## Expected Result

The root validation file should shrink further, the mapped scratch THP gate API should remain available from the crate root, and host plus Docker validation should keep passing.
