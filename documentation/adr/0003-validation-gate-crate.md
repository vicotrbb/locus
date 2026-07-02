# ADR 0003: Separate Validation Gate Crate

Date: 2026-07-02

## Status

Accepted

## Context

Placement validation now has three independently owned evidence paths:

- `locus-sys` owns Linux memory-policy operations and `mbind` readiness.
- `locus-observe` owns Linux locality evidence parsing and placement proof classification.
- `locus-alloc` owns allocator-level mapped arena probes.

A combined placement validation gate must consume outputs from all three paths. Putting that gate in `locus-sys` would make the system boundary depend on observability. Putting it in `locus-observe` would make observability depend on system policy readiness. Putting it in `locus-alloc` would make allocator code responsible for validation orchestration.

## Decision

Create `locus-validate` as a small validation-layer crate. It depends on `locus-sys` and `locus-observe`, and it exposes Linux-gated helpers and examples for combining probe outputs.

## Consequences

- Lower-level crate ownership stays narrow.
- Probe output parsers remain near the evidence they parse.
- Combined validation logic can evolve without creating dependency cycles.
- Host builds remain portable by gating Linux-only validation helpers with `cfg(target_os = "linux")`.
- Successful placement is still not claimed unless the combined gate reports `verified reason=verified`.
