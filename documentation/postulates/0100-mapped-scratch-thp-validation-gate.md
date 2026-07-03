# Postulate 0100: Mapped Scratch THP Validation Gate

Date: 2026-07-03

## Statement

Mapped scratch transparent huge page evidence should have a validation gate that reports readiness only when huge page adoption is observed.

## Rationale

The THP probe and parser distinguish advisory success from observed page-size evidence. Validation tooling should preserve that distinction so automation does not treat `thp_advice=ok` as proof of a larger kernel page size.

A gate can classify the probe output into:

- `ready` only when `hugepage` mode was requested, advice succeeded, and `thp_observed=yes`;
- `unavailable` when the platform or observation evidence is unavailable;
- `not_ready` when advice failed, `no_hugepage` mode was requested, or base pages were observed.

## Experiment

Add `locus-validate` types and functions for `mapped_scratch_thp_validation_gate=<status> reason=<reason>`.

Add a file-based `mapped_scratch_thp_validation_gate` example that reads captured `mapped_scratch_thp` output and prints the final gate line.

## Expected Result

The gate should classify the current Docker output as unavailable because `numa_maps` evidence is unavailable, and it should classify synthetic larger-page evidence as ready.
