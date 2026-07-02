# Postulate 0087: Pinned Scratch Validation Gate

Date: 2026-07-02

## Statement

The pinned scratch pool probe should feed a validation-layer gate that reports whether host page-locked scratch reuse is ready.

## Rationale

The `pinned_scratch_pool` probe and parser make checkout, allocation, release, reuse, and accounting observable. Automation still needs one conservative verdict line, similar to the NUMA placement validation gate, so captured probe output can be classified without duplicating policy checks.

This gate should validate only host page-locked scratch readiness. It must not claim CUDA host registration, GPU-near placement, DMA readiness, or async transfer safety.

## Experiment

Add a `PinnedScratchValidationGate` in `locus-validate` that consumes parsed `PinnedScratchPoolProbeOutput`.

The gate should report:

```text
pinned_scratch_validation_gate=<status> reason=<reason>
```

The gate should be `ready reason=ready` only when:

- first checkout succeeded;
- allocation through the checked-out arena succeeded;
- release succeeded;
- reuse checkout succeeded;
- reuse release succeeded;
- pool stats show at least one idle arena after first release;
- pool stats show at least one reused arena after reuse checkout;
- locked bytes remain non-zero after reuse release.

Add a file-based `pinned_scratch_validation_gate` example that reads captured `pinned_scratch_pool` output and prints the gate line.

## Expected Result

The gate should pass focused unit tests for ready, checkout failure, allocation failure, release failure, missing reuse evidence, parser errors, and gate-line parsing. Docker should run the `pinned_scratch_pool` probe and the validation example successfully.
