# Postulate 0133: Pinned Scratch Near-GPU Probe Parser Module

Date: 2026-07-03

## Statement

The near-GPU pinned scratch probe parser should live in a focused `locus-alloc` module instead of the allocator root file.

## Rationale

The near-GPU parser owns the stable `near_gpu_pool=` constructor line, topology summary fields, selected GPU BDF, arena capacity, locked-byte budget, and the near-GPU interpretation of shared pinned scratch pool event and stats lines. Keeping that parser in `src/lib.rs` after the pool parser extraction leaves the root file dominated by one parser family rather than crate-level API wiring.

Moving the near-GPU parser into `pinned_scratch_near_gpu_probe.rs` should reduce root-file size, keep near-GPU parser invariants near focused tests, and preserve the public API through root re-exports.

## Experiment

Extract the near-GPU pinned scratch parser subsystem into `crates/locus-alloc/src/pinned_scratch_near_gpu_probe.rs`.

The module should own:

- `PinnedScratchNearGpuProbeStatus`;
- `PinnedScratchNearGpuPoolLine`;
- `PinnedScratchNearGpuProbeOutput`;
- `PinnedScratchNearGpuProbeLineParseError`;
- `PinnedScratchNearGpuProbeOutputParseError`;
- `parse_pinned_scratch_near_gpu_probe_pool_line`;
- `parse_pinned_scratch_near_gpu_probe_output`;
- private near-GPU numeric, duplicate-field, required-field, event, and stats helpers;
- focused tests for constructor-line parsing, unavailable output, success output, error output, and malformed output.

The module should depend on the shared pinned scratch pool probe module for event and stats line parsing.

## Real Workload Gate

The extraction must still pass near-GPU validation paths:

- parser unit tests;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- Docker `cargo run -q -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate`.

This exercises the stable near-GPU output parser and the downstream validation crate that depends on the root re-exports.

## Expected Result

The public `locus_alloc::*` near-GPU pinned scratch parser API should remain source compatible, the root allocator file should shrink, near-GPU parser behavior should stay identical, and Docker validation should still classify the container's no-GPU topology as unavailable.
