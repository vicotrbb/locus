# Postulate 0101: Live Mapped Scratch THP Validation Gate

Date: 2026-07-03

## Statement

Mapped scratch transparent huge page evidence should have a live validation gate that emits both probe evidence and the final gate verdict in one command.

## Rationale

The file-based THP validation gate can classify captured probe output, but repeated environment checks are easier and less error-prone when a single example collects the live evidence and immediately evaluates it.

The live gate should preserve the same conservative rule as the file-based gate: accepted advice is not enough for readiness. It should only print `mapped_scratch_thp_validation_gate=ready reason=ready` when live page-size evidence reports `thp_observed=yes`.

## Experiment

Add `live_mapped_scratch_thp_validation_gate` to `locus-validate`.

On Linux, the example should create a mapped scratch arena, apply the requested THP mode, write-touch pages, inspect `numa_maps`, print the stable THP probe lines, then print the validation gate. On non-Linux targets, it should emit the unsupported probe line and the corresponding unavailable gate.

## Expected Result

The current non-Linux host should report `unavailable reason=unsupported_platform`. The current Docker Linux run should report `unavailable reason=observation_unavailable` unless `numa_maps` page-size evidence becomes available and shows larger pages.
