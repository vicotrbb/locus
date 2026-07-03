# Postulate 0094: Pinned Scratch Near-GPU Validation Example

Date: 2026-07-02

## Statement

Captured near-GPU pinned scratch probe output should be classifiable from a command-line example.

## Rationale

The near-GPU validation gate exists as library code, but operational workflows need a small file-based command that can be used with captured output from `pinned_scratch_near_gpu`. This matches the existing `pinned_scratch_validation_gate` workflow for host page-locked scratch output.

## Expected Result

Add a `pinned_scratch_near_gpu_validation_gate` example to `locus-validate` that reads one captured output file and prints `pinned_scratch_near_gpu_validation_gate=<status> reason=<reason>`. It should classify the current host output as `unavailable reason=no_gpu_with_numa_node`.
