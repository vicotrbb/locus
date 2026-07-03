# Experiment 0138: Mapped Scratch THP Probe Parser Module

Date: 2026-07-03

Postulate: [0130 Mapped Scratch THP Probe Parser Module](../postulates/0130-mapped-scratch-thp-probe-parser-module.md)

## Goal

Extract the mapped scratch transparent huge page probe output parser from the broad `locus-alloc` root file into a focused module while preserving the public `locus_alloc::*` parser API and stable Docker probe output.

## Change

Added `crates/locus-alloc/src/mapped_scratch_thp_probe.rs` and moved the THP probe parser subsystem into it:

- `MappedScratchThpProbeRunStatus`;
- `MappedScratchThpAdviceStatus`;
- `MappedScratchThpObservation`;
- `MappedScratchThpProbeOutput`;
- `MappedScratchThpProbeOutputParseError`;
- `parse_mapped_scratch_thp_probe_output`;
- private parser helpers for start lines, advice lines, observation lines, numeric fields, and duplicate detection;
- focused tests for hugepage output, no-hugepage output, unsupported-platform output, advice-error output, malformed output, duplicate fields, and mode mismatch.

The crate root now declares `mod mapped_scratch_thp_probe;` and re-exports the public THP probe parser API. The mapped scratch THP benchmark fault-sample parser remains in `src/lib.rs` for a later benchmark-parser extraction.

## Size Result

| File | Lines |
| --- | ---: |
| `crates/locus-alloc/src/lib.rs` before extraction | 3398 |
| `crates/locus-alloc/src/lib.rs` after extraction | 2736 |
| `crates/locus-alloc/src/mapped_scratch_thp_probe.rs` after extraction | 679 |

The extraction removed 662 lines from the root file while keeping the THP probe parser API available from `locus_alloc::*`.

## Validation Commands

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-alloc --example mapped_scratch_thp -- hugepage
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-alloc --example mapped_scratch_thp -- no_hugepage
```

## Test Results

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `cargo test -p locus-alloc`: passed, 61 tests plus doc tests.

Docker `mapped_scratch_thp -- hugepage` result:

```text
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff9d52f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
```

Docker `mapped_scratch_thp -- no_hugepage` result:

```text
mapped_scratch_thp=started mode=no_hugepage
mapping_start=0xffff7f2bf000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=no_hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
```

## Benchmark Result

No benchmark was added or rerun for this extraction because the change is parser ownership only. It does not alter allocator data structures, transparent huge page advice behavior, mapped memory behavior, or benchmarked allocation paths. The real workload gate is the Docker `mapped_scratch_thp` example in both advice modes plus parser unit tests and downstream validation compilation.

## Conclusion

The postulate survived. The mapped scratch THP probe parser can be owned by a focused module with source-compatible root re-exports, preserved parser behavior, preserved downstream validation compilation, and preserved Docker probe output.

The next root extraction should target the mapped scratch THP fault-sample benchmark parser or the pinned scratch pool probe parser.
