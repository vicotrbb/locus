# Postulate 0095: Live Pinned Scratch Near-GPU Validation Gate

Date: 2026-07-02

## Statement

Near-GPU pinned scratch validation should have a live command that prints probe evidence and the final gate verdict in one run.

## Rationale

The file-based `pinned_scratch_near_gpu_validation_gate` example is useful for captured output, but host qualification and CI workflows benefit from a direct command that discovers topology, attempts the near-GPU pinned scratch pool path, and evaluates the exact stable lines it printed.

This remains host topology and page-lock validation only. It does not register memory with CUDA, select a CUDA device, prove DMA behavior, or prove final page placement.

## Expected Result

Add `live_pinned_scratch_near_gpu_validation_gate` to `locus-validate`. It should accept an optional GPU BDF, print the same stable probe lines as `pinned_scratch_near_gpu`, then print `pinned_scratch_near_gpu_validation_gate=<status> reason=<reason>`.

On the current host and Docker environment, it should report `unavailable reason=no_gpu_with_numa_node`.
