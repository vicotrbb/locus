# Postulate 0021: Mapped Arena Placement Identity

Date: 2026-07-02

## Statement

A mapped scratch arena should expose stable mapping identity so placement probes can correlate the arena under test with `/proc/self/numa_maps` evidence.

## Rationale

The current bind probe reports policy attempt status and page touches, but it does not print the address range being tested. Without a mapping address, a successful future run cannot prove that a specific arena corresponds to a specific `numa_maps` row. A safe address and length accessor keeps the unsafe mapping boundary inside `locus-sys` while giving validation tools enough identity to inspect page placement.

## Experiment

Add:

- safe `MappedScratchArena` mapping start and length accessors;
- an exact-start helper for parsed `numa_maps` entries;
- focused tests for both helper surfaces;
- mapped scratch bind probe output for mapping start and length.

## Expected Result

The helper APIs should pass workspace tests and clippy. In the current Docker environment the bind probe should still fail with `EPERM`, but it should print mapping identity before reporting the policy failure.
