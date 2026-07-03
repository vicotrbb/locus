# Experiment 0157: THP Benchmark Page-Size Samples

Date: 2026-07-03

## Postulate

[Postulate 0149](../postulates/0149-thp-benchmark-page-size-samples.md)
claimed that the mapped scratch THP benchmark should emit a small page-size
evidence sample for each advice mode before Criterion timing so benchmark logs
can join timing, fault counters, and kernel page-size evidence from the same
run.

## Change

Added `crates/locus-alloc/src/mapped_scratch_thp_page_sample.rs` with typed
parsers for:

- `thp_page_sample=` lines;
- complete benchmark output containing `default`, `hugepage`, and
  `no_hugepage` page samples.

Updated `crates/locus-alloc/benches/scratch_arena.rs` so the Linux
`mapped_scratch_write_touch_4mib_*` benchmark prints one page-size sample per
mode before existing fault samples and Criterion timing.

Each page sample:

- creates a real `MappedScratchArena`;
- applies the requested advice mode;
- write-touches the mapping;
- tries `numa_maps` evidence first;
- falls back to `smaps`;
- emits source, kernel page size, observation, and reason fields.

## Validation

Host commands:

```bash
cargo test -p locus-alloc mapped_scratch_thp_page_sample
cargo bench -p locus-alloc --bench scratch_arena --no-run
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10 --warm-up-time 1 --measurement-time 1 2>&1 | tee /tmp/locus-thp-bench.out; sample_count=$(grep -c "^thp_page_sample=" /tmp/locus-thp-bench.out); echo thp_page_sample_count=$sample_count; test "$sample_count" -eq 3'
```

Host results:

- `cargo test -p locus-alloc mapped_scratch_thp_page_sample`: passed, 5
  focused tests.
- `cargo bench -p locus-alloc --bench scratch_arena --no-run`: passed.
- `cargo test --workspace`: passed, 176 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Docker page-size samples:

```text
thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample_count=3
```

Docker fault samples from the same benchmark log:

```text
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
```

Docker Criterion timing summary:

| Benchmark | Timing |
| --- | ---: |
| `mapped_scratch_write_touch_4mib_default` | 878.54 us to 1.4043 ms |
| `mapped_scratch_write_touch_4mib_hugepage_advice` | 31.610 us to 32.391 us |
| `mapped_scratch_write_touch_4mib_no_hugepage_advice` | 804.15 us to 818.20 us |

Criterion printed change percentages from mounted workspace history. Those
percentages are not used here because they compare across contexts.

## Interpretation

The postulate survived.

The benchmark now emits parseable page-size evidence, fault samples, and timing
in one run. In Docker, all three advice modes found `smaps` evidence and all
three sampled mappings reported 4 KiB kernel pages. The hugepage advice timing
was still much faster in this short run, but the page-size sample is negative
evidence for huge page adoption in this environment.

The timing remains useful as a lead, but it is not proof that the measured loop
used huge pages. Future THP reports should use these page samples to reject
advice-only interpretations.

## Next Step

Add a validation report that parses both `thp_page_sample=` and `fault_sample=`
lines from one benchmark output and emits a single THP benchmark evidence
summary.
