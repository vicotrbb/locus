# Postulate 0014: Linux Mbind Wrapper

Date: 2026-07-02

## Statement

Locus needs a narrow Linux `mbind` wrapper in `locus-sys` before mapped allocator experiments can apply explicit NUMA placement policies.

## Rationale

The allocator crates should not construct NUMA masks or call Linux syscalls directly. A focused wrapper keeps policy application inside the system boundary and lets higher-level code pass owned mapped regions to a safe API.

## Experiment

Add a Linux-only `bind_region_to_node` helper that:

- accepts a `MappedRegion`;
- constructs a single-node mask;
- calls `mbind` with `MPOL_BIND`;
- reports invalid node masks and syscall failures;
- tests node-mask construction in the Linux build.

## Expected Result

The wrapper should compile locally through conditional compilation and pass Linux container tests for mask construction. It should not yet be wired into allocator experiments or claim placement validation.
