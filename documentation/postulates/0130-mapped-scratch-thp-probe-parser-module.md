# Postulate 0130: Mapped Scratch THP Probe Parser Module

Date: 2026-07-03

## Statement

The mapped scratch transparent huge page probe output parser should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

The THP probe parser is a stable text interface for `mapped_scratch_thp` output. Keeping its run status, advice status, observation schema, parsed output type, parse error, parser helpers, and tests in `src/lib.rs` mixes one parser family with pinned scratch pool parsing, near-GPU parsing, and THP benchmark fault-sample parsing.

Moving the probe parser into `mapped_scratch_thp_probe.rs` should reduce root-file size, keep the probe schema near its tests, and preserve the public API through root re-exports.

## Experiment

Extract the mapped scratch THP probe parser subsystem into `crates/locus-alloc/src/mapped_scratch_thp_probe.rs`.

The module should own:

- `MappedScratchThpProbeRunStatus`;
- `MappedScratchThpAdviceStatus`;
- `MappedScratchThpObservation`;
- `MappedScratchThpProbeOutput`;
- `MappedScratchThpProbeOutputParseError`;
- `parse_mapped_scratch_thp_probe_output`;
- private parser helpers for start, advice, observation, numeric fields, and duplicate detection;
- focused tests for hugepage output, no-hugepage output, unsupported-platform output, advice-error output, malformed output, duplicate fields, and mode mismatch.

The mapped scratch THP benchmark fault-sample parser should remain in the root for this experiment. It can move later as a separate benchmark-parser extraction.

## Real Workload Gate

The extraction must still pass existing THP probe validation paths:

- parser unit tests;
- `mapped_scratch_thp` example in Docker;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets -- -D warnings`.

These exercise stable probe output compatibility and downstream validation crates that depend on the root re-exports.

## Expected Result

The public `locus_alloc::*` THP probe parser API should remain source compatible, the root allocator file should shrink, parser behavior should stay identical, and Docker should still produce stable `mapped_scratch_thp` output.
