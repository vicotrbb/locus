# Postulate 0088: Live Pinned Scratch Validation Gate

Date: 2026-07-02

## Statement

The host page-locked scratch validation gate should have a live one-command example.

## Rationale

The file-based `pinned_scratch_validation_gate` example is useful for captured probe output, but the working loop also benefits from a direct live command like `live_placement_validation_gate`. A live pinned scratch gate should run the same checkout, allocation, release, and reuse sequence, then evaluate the exact stable lines it printed.

This remains a host page-lock validation path. It does not prove CUDA host registration, GPU-near placement, DMA readiness, or async transfer safety.

## Experiment

Add a `live_pinned_scratch_validation_gate` example to `locus-validate`.

The example should:

- create a budgeted `PinnedScratchPool`;
- print the same stable lines as the `pinned_scratch_pool` probe;
- evaluate the captured stable output with `evaluate_pinned_scratch_validation_output`;
- print `pinned_scratch_validation_gate=<status> reason=<reason>`.

## Expected Result

The example should compile and run on the host and in Docker. In the current Docker profile, it should report `pinned_scratch_validation_gate=ready reason=ready`.
