# Postulate 0118: Linux Placement Gate Module

Date: 2026-07-03

## Statement

The Linux placement validation gate should live in a Linux submodule instead of the `locus-validate` crate root.

## Rationale

The root validation file now mainly coordinates validation modules, but it still owns the Linux placement validation gate implementation. That gate combines Linux memory-policy readiness, locality evidence readiness, mapped arena placement proof parsing, multiline gate parsing, and platform-specific tests.

Moving the Linux gate into `linux/placement_validation_gate.rs`, with `linux/mod.rs` as the public Linux API surface, should keep the crate root compact and make the platform boundary explicit without changing the existing `locus_validate::linux::*` API.

## Experiment

Extract the Linux placement validation gate into `crates/locus-validate/src/linux/placement_validation_gate.rs` and re-export it from `crates/locus-validate/src/linux/mod.rs`.

The module should own:

- placement validation input, gate, verdict, status, and reason types;
- line, output, and probe parse errors;
- display implementations;
- combined evaluation from Linux memory policy, placement readiness, and placement proof outputs;
- stable gate line and multiline output parsing;
- focused Linux-only tests.

## Expected Result

The root validation file should become a compact crate-level API surface, the Linux API should remain source compatible, and host plus Docker validation should keep passing.
