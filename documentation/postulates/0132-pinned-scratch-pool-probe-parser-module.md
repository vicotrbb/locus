# Postulate 0132: Pinned Scratch Pool Probe Parser Module

Date: 2026-07-03

## Statement

The pinned scratch pool probe parser should live in a focused `locus-alloc` module instead of the allocator root file.

## Rationale

Pinned scratch pool probe parsing owns a stable text interface for pool checkout, allocation, release, reuse, and stats lines. It also provides the shared event and stats parser used by near-GPU pinned scratch validation. Keeping these types, parser errors, helper functions, and tests in `src/lib.rs` makes the root file harder to review and mixes two parser families in one broad file.

Moving the pool probe parser into `pinned_scratch_pool_probe.rs` should reduce root-file size, keep pool probe invariants near focused tests, and leave near-GPU parsing able to depend on the shared pool line parser.

## Experiment

Extract the pinned scratch pool probe parser subsystem into `crates/locus-alloc/src/pinned_scratch_pool_probe.rs`.

The module should own:

- `PinnedScratchPoolProbeStatus`;
- `PinnedScratchPoolProbeEvent`;
- `PinnedScratchPoolProbePhase`;
- `PinnedScratchPoolProbeEventLine`;
- `PinnedScratchPoolProbeStatsLine`;
- `PinnedScratchPoolProbeOutput`;
- `PinnedScratchPoolProbeLineParseError`;
- `PinnedScratchPoolProbeOutputParseError`;
- `parse_pinned_scratch_pool_probe_event_line`;
- `parse_pinned_scratch_pool_probe_stats_line`;
- `parse_pinned_scratch_pool_probe_output`;
- crate-private pool event and stats line classifiers used by near-GPU parsing;
- private numeric, duplicate-field, required-field, and output aggregation helpers;
- focused tests for event parsing, stats parsing, output aggregation, checkout-error output, and malformed output.

## Real Workload Gate

The extraction must still pass pinned scratch validation paths:

- parser unit tests;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- Docker `cargo run -q -p locus-validate --example live_pinned_scratch_validation_gate`;
- Docker `cargo run -q -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate`.

These exercise the stable probe lines and downstream validation crates that depend on the root re-exports.

## Expected Result

The public `locus_alloc::*` pinned scratch pool probe API should remain source compatible, near-GPU parsing should still consume the shared pool event and stats lines, root allocator source should shrink, and Docker validation should still produce parseable pinned scratch gate output.
