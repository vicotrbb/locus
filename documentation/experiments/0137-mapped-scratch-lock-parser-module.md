# Experiment 0137: Mapped Scratch Lock Parser Module

Date: 2026-07-03

Postulate: [0129 Mapped Scratch Lock Parser Module](../postulates/0129-mapped-scratch-lock-parser-module.md)

## Goal

Extract the mapped scratch page-lock probe parser from the broad `locus-alloc` root file into a focused module while preserving the public `locus_alloc::*` parser API and stable Docker probe output.

## Change

Added `crates/locus-alloc/src/mapped_scratch_lock_probe.rs` and moved the page-lock parser subsystem into it:

- `PageLockProbeStatus`;
- `PageLockProbeField`;
- `PageLockProbeStatusLine`;
- `MappedScratchLockProbeOutput`;
- `PageLockProbeStatusLineParseError`;
- `MappedScratchLockProbeOutputParseError`;
- `parse_page_lock_probe_status_line`;
- `parse_mapped_scratch_lock_probe_output`;
- focused tests for valid status lines, invalid status lines, valid probe output, and invalid probe output.

The crate root now declares `mod mapped_scratch_lock_probe;` and re-exports the public page-lock parser API. THP, pinned scratch pool, and near-GPU parser families remain in `src/lib.rs` for later focused extractions.

## Size Result

| File | Lines |
| --- | ---: |
| `crates/locus-alloc/src/lib.rs` before extraction | 3741 |
| `crates/locus-alloc/src/lib.rs` after extraction | 3398 |
| `crates/locus-alloc/src/mapped_scratch_lock_probe.rs` after extraction | 355 |

The extraction removed 343 lines from the root file while keeping the page-lock parser API available from `locus_alloc::*`.

## Validation Commands

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-alloc --example mapped_scratch_lock
```

## Test Results

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `cargo test -p locus-alloc`: passed, 61 tests plus doc tests.

Docker `mapped_scratch_lock` result:

```text
mapping_start=0xffffaee81000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
```

## Benchmark Result

No benchmark was added or rerun for this extraction because the change is parser ownership only. It does not alter allocator data structures, page-locking behavior, mapped memory behavior, or benchmarked allocation paths. The real workload gate is the Docker `mapped_scratch_lock` example plus parser unit tests and downstream validation compilation.

## Conclusion

The postulate survived. The mapped scratch page-lock parser can be owned by a focused module with source-compatible root re-exports, preserved parser behavior, preserved downstream compilation, and preserved Docker probe output.

The next root extraction should target the mapped scratch THP parser or pinned scratch pool probe parser as a separate parser module.
