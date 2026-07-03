# Postulate 0116: Pinned Scratch Gate Module

Date: 2026-07-03

## Statement

The host page-locked pinned scratch validation gate should live in a focused `locus-validate` module instead of the broad validation root file.

## Rationale

The root validation file still owns the base pinned scratch validation gate and the near-GPU pinned scratch validation gate. The base pinned scratch gate is a standalone validation boundary around `PinnedScratchPoolProbeOutput`, while the near-GPU gate has separate topology and availability semantics.

Moving the base pinned scratch gate into `pinned_scratch_gate.rs` should reduce root file size, isolate stable line parsing near the base gate model, and leave the near-GPU gate available for a later focused extraction.

## Experiment

Extract the base pinned scratch validation gate into `crates/locus-validate/src/pinned_scratch_gate.rs`.

The module should own:

- gate status, reason, gate, and verdict types;
- line, output, and evaluation parse errors;
- display implementations;
- probe evaluation from parsed pinned scratch pool output;
- line and multiline gate parsers;
- focused tests for ready, not-ready, parser-error, duplicate, malformed, and inconsistent cases.

## Expected Result

The root validation file should shrink further, the base pinned scratch gate API should remain available from the crate root, and host plus Docker validation should keep passing.
