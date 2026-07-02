# Postulate 0063: Combined Placement Validation Gate

Date: 2026-07-02

## Statement

Locus should provide a small validation-layer helper that combines memory-policy readiness, locality evidence readiness, and mapped arena placement proof into one conservative verdict.

## Rationale

The individual probes now emit stable final lines and parser helpers exist for full probe output. Future validation automation still needs to apply the acceptance rule consistently:

- memory policy readiness must be ready;
- placement evidence readiness must be ready;
- placement proof must be verified.

A separate validation crate can depend on both `locus-sys` and `locus-observe` without creating ownership or dependency cycles in the lower-level crates.

## Experiment

Add a `locus-validate` crate with a Linux-gated helper that parses the three probe outputs and returns a typed combined placement validation gate verdict.

## Expected Result

The helper should pass focused Linux tests, compile on the host through conditional compilation, and report the current Docker outputs as not ready rather than verified placement.
