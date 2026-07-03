# Postulate 0131: Mapped Scratch THP Fault Sample Parser Module

Date: 2026-07-03

## Statement

The mapped scratch THP benchmark fault-sample parser should live in a focused `locus-alloc` module instead of the broad allocator root file.

## Rationale

The fault-sample parser is a stable text interface for Criterion benchmark output lines such as `fault_sample=default`. It owns benchmark evidence parsing, complete sample aggregation, comparison calculation, parser errors, and tests. Keeping it in `src/lib.rs` mixes benchmark-output parsing with pinned scratch pool and near-GPU parser families.

Moving the fault-sample parser into `mapped_scratch_thp_fault_sample.rs` should reduce root-file size, keep benchmark evidence parsing near its tests, and preserve the public API through root re-exports.

## Experiment

Extract the mapped scratch THP fault-sample parser subsystem into `crates/locus-alloc/src/mapped_scratch_thp_fault_sample.rs`.

The module should own:

- `MappedScratchThpFaultSampleMode`;
- `MappedScratchThpFaultSampleStatus`;
- `MappedScratchThpFaultSampleLine`;
- `MappedScratchThpFaultSamples`;
- `MappedScratchThpFaultSampleComparison`;
- `MappedScratchThpFaultSampleLineParseError`;
- `MappedScratchThpFaultSamplesParseError`;
- `parse_mapped_scratch_thp_fault_sample_line`;
- `parse_mapped_scratch_thp_fault_samples_output`;
- private helpers for numeric parsing, duplicate-field detection, sample aggregation, and comparison deltas;
- focused tests for line parsing, sample aggregation, comparison, incomplete comparison, major-fault detection, malformed lines, and malformed sample output.

## Real Workload Gate

The extraction must still pass existing fault-sample validation paths:

- parser unit tests;
- `cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib` in Docker;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets -- -D warnings`.

These exercise stable benchmark evidence lines and downstream validation crates that depend on the root re-exports.

## Expected Result

The public `locus_alloc::*` THP fault-sample parser API should remain source compatible, the root allocator file should shrink, parser behavior should stay identical, and Docker benchmark output should still include parseable fault-sample lines.
