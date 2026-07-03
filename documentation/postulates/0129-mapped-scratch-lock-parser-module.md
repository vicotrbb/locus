# Postulate 0129: Mapped Scratch Lock Parser Module

Date: 2026-07-03

## Statement

The mapped scratch page-lock probe parser should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

The page-lock parser is a stable text interface for `mapped_scratch_lock` output. Keeping its status enums, parsed output type, parse errors, parser functions, and tests in `src/lib.rs` mixes parser schema code with unrelated pinned scratch, near-GPU, THP, and fault-sample parser families.

Moving the page-lock parser into `mapped_scratch_lock_probe.rs` should reduce root-file size, keep the parser schema near its tests, and preserve the public API through root re-exports.

## Experiment

Extract the mapped scratch page-lock parser subsystem into `crates/locus-alloc/src/mapped_scratch_lock_probe.rs`.

The module should own:

- `PageLockProbeStatus`;
- `PageLockProbeField`;
- `PageLockProbeStatusLine`;
- `MappedScratchLockProbeOutput`;
- `PageLockProbeStatusLineParseError`;
- `MappedScratchLockProbeOutputParseError`;
- `parse_page_lock_probe_status_line`;
- `parse_mapped_scratch_lock_probe_output`;
- focused parser tests for valid status lines, invalid status lines, valid probe output, and invalid probe output.

## Real Workload Gate

The extraction must still pass existing mapped scratch lock validation paths:

- parser unit tests;
- `mapped_scratch_lock` example in Docker;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets -- -D warnings`.

These exercise stable probe output compatibility and make sure downstream validation crates still compile through root re-exports.

## Expected Result

The public `locus_alloc::*` page-lock parser API should remain source compatible, the root allocator file should shrink, parser behavior should stay identical, and Docker should still produce stable `mapped_scratch_lock` output.
