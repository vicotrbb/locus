# Experiment 0112: THP Benchmark Fault Counter Sample

Date: 2026-07-03

## Postulate

[Postulate 0104: THP Benchmark Fault Counter Sample](../postulates/0104-thp-benchmark-fault-counter-sample.md)

## Change

Added Linux-only process fault counter sampling to the mapped scratch THP write-touch benchmark.

The benchmark now prints one `fault_sample=` line for each mode before Criterion timing begins:

- `default`;
- `hugepage`;
- `no_hugepage`.

Each sample runs eight benchmark-like iterations and reports signed deltas for process and child minor and major fault counters. The Criterion benchmark names are unchanged.

## Commands

```text
cargo fmt --all
cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
cargo fmt --all -- --check
git diff --check
rg "$(printf '\342\200\224')" README.md documentation crates Cargo.toml Cargo.lock
```

## Results

The host benchmark command compiled the harness. The THP benchmark is Linux-gated, so the non-Linux host reported no matching timing output.

Docker benchmark fault sample output:

```text
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
```

Docker Criterion timing output:

```text
mapped_scratch_write_touch_4mib_default
                        time:   [680.49 us 685.13 us 691.25 us]
mapped_scratch_write_touch_4mib_hugepage_advice
                        time:   [26.775 us 27.036 us 27.411 us]
mapped_scratch_write_touch_4mib_no_hugepage_advice
                        time:   [681.63 us 690.14 us 696.67 us]
```

Full validation passed:

- `cargo test --workspace`: 127 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `cargo test -p locus-alloc`: 53 unit tests passed, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- em dash scan: no matches.

## Conclusion

The postulate survived. The benchmark now emits page-fault context before timing the three THP modes. In this Docker run, the `hugepage` advice sample had about half the process minor fault delta of the default and `no_hugepage` samples, while major fault deltas stayed at zero.

This is still supporting evidence, not proof of huge page adoption. The sample is process-wide and short; it should be interpreted alongside `numa_maps` page-size evidence and repeated benchmark runs.

## Next Questions

- Should the benchmark write fault samples to a separate machine-readable artifact to avoid mixing diagnostics with Criterion output?
- Should the THP validation gate and benchmark share a combined record type for timing, fault deltas, and page-size evidence?
