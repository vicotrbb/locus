# Experiment 0135: Mapped Scratch Module

Date: 2026-07-03

Postulate: [0127 Mapped Scratch Module](../postulates/0127-mapped-scratch-module.md)

## Goal

Extract the mmap-backed mapped scratch arena from the broad `locus-alloc` root file into a focused module while preserving the public `locus_alloc::*` API and the mapped allocation validation surface.

## Change

Added `crates/locus-alloc/src/mapped_scratch.rs` and moved the mapped scratch arena subsystem into it:

- `MappedScratchArena`;
- `MappedScratchAllocError`;
- `MappedScratchHugePageAdvice`;
- mapped arena alignment rounding;
- focused mapped arena tests for alignment, reset accounting, out-of-memory behavior, page touching, page locking, mapping identity, Linux bind rejection, and transparent huge page advice.

The crate root now declares `mod mapped_scratch;` and re-exports the public mapped scratch API. The pinned scratch pool, probe output types, and parser logic remain in `src/lib.rs`.

## Size Result

| File | Lines |
| --- | ---: |
| `crates/locus-alloc/src/lib.rs` before extraction | 4724 |
| `crates/locus-alloc/src/lib.rs` after extraction | 4287 |
| `crates/locus-alloc/src/mapped_scratch.rs` after extraction | 456 |

The root file is still too large, but the extraction removed 437 lines from it without changing the public mapped scratch API path.

## Validation Commands

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-alloc --example mapped_scratch_lock
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-alloc --example mapped_scratch_thp -- hugepage
rm -rf /tmp/locus-mapped-bench-target
CARGO_TARGET_DIR=/tmp/locus-mapped-bench-target cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_arena_reset_cycle_64x256b --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-mapped-bench-target cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_1mib --sample-size 10 --warm-up-time 1 --measurement-time 2
CARGO_TARGET_DIR=/tmp/locus-mapped-bench-target cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 2
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 2
```

## Test Results

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `cargo test -p locus-alloc`: passed, 61 tests plus doc tests.

Docker `mapped_scratch_lock` result:

```text
mapping_start=0xffffa1be5000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
```

Docker `mapped_scratch_thp -- hugepage` result:

```text
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff9ccaf000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
```

## Benchmark Results

Host mapped scratch benchmarks:

| Benchmark | Result |
| --- | ---: |
| `mapped_scratch_arena_reset_cycle_64x256b` | 202.96 ns to 206.43 ns |
| `mapped_scratch_write_touch_1mib` | 34.056 us to 34.580 us |

The host `mapped_scratch_write_touch_4mib` run produced no timing output on macOS because the Linux THP comparison benchmark is a no-op outside Linux.

Docker Linux mapped scratch 4 MiB write-touch benchmark:

| Benchmark | Result |
| --- | ---: |
| `mapped_scratch_write_touch_4mib_default` | 672.87 us to 685.42 us |
| `mapped_scratch_write_touch_4mib_hugepage_advice` | 27.610 us to 28.686 us |
| `mapped_scratch_write_touch_4mib_no_hugepage_advice` | 673.11 us to 686.14 us |

Linux fault sample before the timed runs:

```text
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
```

This repeats the key mapped scratch THP result: hugepage advice roughly halves minor faults in the sample and is much faster than default or no-hugepage advice for the first-touch benchmark. The latest range did not beat the current best result in `documentation/dev-notes/2026-07-03-best-benchmark-results.md`, so that best-results note was not changed.

## Conclusion

The postulate survived. `MappedScratchArena` can be owned by a focused module with source-compatible root re-exports, preserved unit coverage, preserved Docker validation, and preserved Linux mapped scratch benchmark behavior.

The next allocator-root extraction should target pinned scratch pool or mapped scratch probe parsing, since `src/lib.rs` still contains more than four thousand lines.
