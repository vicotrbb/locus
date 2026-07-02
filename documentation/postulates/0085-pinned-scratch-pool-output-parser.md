# Postulate 0085: Pinned Scratch Pool Output Parser

Date: 2026-07-02

## Statement

The pinned scratch pool probe should have a typed parser for its stable event and accounting lines.

## Rationale

The `pinned_scratch_pool` example now prints stable checkout, allocation, release, reuse, and pool accounting lines. Automation should consume those lines through a shared parser instead of string matching in downstream scripts.

Parsing both event lines and accounting lines lets future validation distinguish a successful reuse proof from a page-lock failure, missing release, or duplicate output.

This remains a host page-lock readiness parser. It does not prove CUDA registration, GPU-near placement, or async transfer safety.

## Experiment

Add parser types and functions in `locus-alloc` for:

- stable pinned scratch pool event lines;
- stable pinned scratch pool stats lines;
- full multiline `pinned_scratch_pool` output.

The parser should:

- require a `pool_checkout` line;
- allow checkout failure output with `pool_checkout=error`;
- require allocation, release, reuse checkout, reuse release, and phase stats when checkout succeeds;
- reject duplicate stable lines;
- reject unknown status, phase, field, or numeric tokens.

## Expected Result

The parser should pass focused unit tests and keep the workspace gates green. Docker should still run the `pinned_scratch_pool` example successfully.
