# Postulate 0117: Pinned Scratch Near-GPU Gate Module

Date: 2026-07-03

## Statement

The near-GPU pinned scratch validation gate should live in a focused `locus-validate` module instead of the broad validation root file.

## Rationale

The near-GPU gate has distinct semantics from the base pinned scratch gate. It combines GPU PCI locality discovery, unavailable-environment classification, page-locked pool construction, checkout, allocation, release, and pool accounting checks.

Keeping that behavior in the root validation file makes the crate root harder to scan and mixes GPU-local validation details with Linux placement validation. Moving the near-GPU gate into `pinned_scratch_near_gpu_gate.rs` should preserve API compatibility through root re-exports while isolating the topology-aware gate model and parser tests.

## Experiment

Extract the near-GPU pinned scratch validation gate into `crates/locus-validate/src/pinned_scratch_near_gpu_gate.rs`.

The module should own:

- gate status, reason, gate, and verdict types;
- line and evaluation parse errors;
- display implementations;
- probe evaluation from parsed near-GPU pinned scratch output;
- stable gate line parsing;
- focused tests for ready, unavailable, not-ready, accounting-failure, parser-error, and inconsistent cases.

## Expected Result

The root validation file should shrink to re-exports plus Linux placement validation, the near-GPU gate API should remain available from the crate root, and host plus Docker validation should keep passing.
