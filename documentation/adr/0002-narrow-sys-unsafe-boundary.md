# ADR 0002: Narrow System Unsafe Boundary

Date: 2026-07-02

## Status

Accepted

## Context

Locus has so far kept allocator experiments safe and Vec-backed. To validate real memory placement, the project needs owned memory mappings, Linux memory policy calls, and eventually pinned host memory registration. Those operations require unsafe code and direct operating-system calls.

Keeping unsafe code spread across allocator crates would make review harder and weaken the project invariant that Rust code stays safe by default.

## Decision

Introduce `locus-sys` as the explicit system boundary. The workspace keeps `unsafe_code = forbid` by default, and `locus-sys` overrides that lint locally. Public APIs in `locus-sys` should remain safe and owned whenever possible.

The first primitive is an owned anonymous `MappedRegion` built with `mmap` and released with `munmap`.

## Consequences

- Unsafe code is isolated in one crate with focused tests.
- Allocator crates can consume owned memory primitives without calling syscalls directly.
- Future Linux NUMA calls can be added behind the same boundary.
- Reviews can focus extra scrutiny on `locus-sys`.
