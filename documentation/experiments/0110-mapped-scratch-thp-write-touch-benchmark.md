# Experiment 0110: Mapped Scratch THP Write-Touch Benchmark

Date: 2026-07-03

## Postulate

[Postulate 0102: Mapped Scratch THP Write-Touch Benchmark](../postulates/0102-mapped-scratch-thp-write-touch-benchmark.md)

## Change

Extended the `scratch_arena` Criterion harness with Linux-only 4 MiB mapped scratch write-touch cases:

- `mapped_scratch_write_touch_4mib_default`;
- `mapped_scratch_write_touch_4mib_hugepage_advice`;
- `mapped_scratch_write_touch_4mib_no_hugepage_advice`.

The benchmark function is a no-op on non-Linux targets so the harness still compiles on the host.

## Commands

```text
cargo fmt --all
cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

The host benchmark command compiled the harness, but the new THP benchmark function is Linux-gated and therefore ran no matching benchmark on the non-Linux host.

Workspace tests passed. Clippy passed with `-D warnings`.

Docker focused tests passed:

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Docker benchmark output:

```text
mapped_scratch_write_touch_4mib_default
                        time:   [675.72 us 683.47 us 695.21 us]
mapped_scratch_write_touch_4mib_hugepage_advice
                        time:   [27.359 us 27.520 us 27.781 us]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
mapped_scratch_write_touch_4mib_no_hugepage_advice
                        time:   [682.37 us 689.76 us 694.57 us]
```

## Conclusion

The postulate survived. The THP advice path now has benchmark coverage for first-touch cost.

This short Docker sample is baseline evidence only. It shows that the `hugepage` advice case behaved differently in this container run, but it does not prove a general performance result without larger sample sizes, repeated runs, and live page-size evidence from the THP validation gate.
