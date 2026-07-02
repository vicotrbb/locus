# Postulate 0084: Pinned Scratch Pool Probe

Date: 2026-07-02

## Statement

The pinned scratch pool should have a small command-line probe that exercises checkout, allocation, release, and reuse with stable output.

## Rationale

Unit tests validate pool behavior, but future validation gates need executable probes that can be run under Docker and host environments. A probe should make page-lock availability and pool accounting visible without requiring test harness output.

This remains a host page-lock probe. It does not prove CUDA registration, GPU-near placement, or async transfer safety.

## Experiment

Add a `pinned_scratch_pool` example that:

- creates a budgeted `PinnedScratchPool`;
- prints pool configuration;
- checks out one arena;
- allocates through the checked-out arena;
- releases the arena;
- checks out again to prove idle reuse;
- prints pool stats after each phase;
- reports checkout errors as stable status lines.

## Expected Result

The example should compile and run in Docker. In the current Docker profile, the small page-locked arena is expected to check out successfully and show reuse with one created arena.
