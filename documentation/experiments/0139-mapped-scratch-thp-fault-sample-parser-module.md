# Experiment 0139: Mapped Scratch THP Fault Sample Parser Module

Date: 2026-07-03

Postulate: `documentation/postulates/0131-mapped-scratch-thp-fault-sample-parser-module.md`

## Question

Can the mapped scratch THP fault-sample parser move out of `crates/locus-alloc/src/lib.rs` into a focused module while preserving the public `locus_alloc::*` API, parser behavior, downstream validation behavior, and real benchmark evidence output?

## Change

Moved the mapped scratch THP fault-sample parser subsystem into:

- `crates/locus-alloc/src/mapped_scratch_thp_fault_sample.rs`

The new module owns:

- `MappedScratchThpFaultSampleMode`;
- `MappedScratchThpFaultSampleStatus`;
- `MappedScratchThpFaultSampleLine`;
- `MappedScratchThpFaultSamples`;
- `MappedScratchThpFaultSampleComparison`;
- `MappedScratchThpFaultSampleLineParseError`;
- `MappedScratchThpFaultSamplesParseError`;
- `parse_mapped_scratch_thp_fault_sample_line`;
- `parse_mapped_scratch_thp_fault_samples_output`;
- private numeric parsing, duplicate-field detection, sample aggregation, and comparison helpers;
- focused parser and comparison tests.

`crates/locus-alloc/src/lib.rs` now keeps the root API source-compatible through `pub use` re-exports.

## Size Result

| File | Lines before | Lines after |
| --- | ---: | ---: |
| `crates/locus-alloc/src/lib.rs` | 2736 | 1868 |
| `crates/locus-alloc/src/mapped_scratch_thp_fault_sample.rs` | 0 | 882 |

Root `lib.rs` shrank by 868 lines while the total allocator source size changed only by formatting and module boilerplate.

## Validation

Commands:

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 2 > /tmp/locus-docker-fault-sample-bench.txt 2>&1
rg -n "fault_sample|mapped_scratch_write_touch_4mib|time:" /tmp/locus-docker-fault-sample-bench.txt
```

Results:

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed, 153 unit tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker Criterion run: passed and emitted parseable fault-sample lines.

Docker fault-sample evidence:

```text
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
```

Docker timing evidence:

| Benchmark | Observed interval |
| --- | ---: |
| `mapped_scratch_write_touch_4mib_default` | 820.39 us to 919.53 us |
| `mapped_scratch_write_touch_4mib_hugepage_advice` | 29.660 us to 32.230 us |
| `mapped_scratch_write_touch_4mib_no_hugepage_advice` | 895.81 us to 940.76 us |

This confirms the real benchmark still emits the stable fault-sample lines after the parser extraction. The Docker timing did not beat the current best THP-advised mapped scratch result of 27.359 us to 27.781 us, so `documentation/dev-notes/2026-07-03-best-benchmark-results.md` was not updated.

## Conclusion

Postulate 0131 survived.

The parser now has a focused module and focused tests, the root API remains source-compatible, downstream validation still passes, and the Docker benchmark still produces parseable evidence lines. This is a maintainability improvement with no measured behavior regression in the tested parser and benchmark paths.
