# Postulate 0092: Pinned Scratch Near-GPU Output Parser

Date: 2026-07-02

## Statement

The near-GPU pinned scratch probe should have a parser for its stable machine-readable output.

## Rationale

`pinned_scratch_near_gpu` now reports topology counts, the selected GPU BDF when available, constructor status, checkout behavior, allocation behavior, release behavior, and pool stats. Without a parser, downstream validation gates must duplicate line handling or rely on brittle text checks.

The parser should classify the three constructor outcomes:

- `near_gpu_pool=ok`;
- `near_gpu_pool=unavailable`;
- `near_gpu_pool=error`.

For successful constructor output, it should parse the reduced checkout path used by the near-GPU probe. This path requires initial stats and checkout, then requires allocation, release, and after-checkout stats only when checkout succeeds.

## Expected Result

Unit tests should parse successful, unavailable, and constructor-error outputs, reject duplicate or malformed stable lines, and pass under workspace tests and clippy.
