# Experiment 0028: First Touch Materialization Benchmark

Date: 2026-07-02

## Postulate

See `documentation/postulates/0020-first-touch-materialization-benchmark.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 mapped_scratch_write_touch_1mib
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1 vec_write_touch_1mib
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 10 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Focused benchmark output:

```text
mapped_scratch_write_touch_1mib
                        time:   [36.140 us 36.437 us 36.762 us]

vec_write_touch_1mib    time:   [1.8701 us 1.9650 us 2.0037 us]
```

The mapped benchmark creates an anonymous mapping, then write-touches one byte per page. The `Vec<u8>` benchmark allocates a 1 MiB vector through the default allocator path and writes one byte per 4 KiB page.

## Conclusion

The postulate survived. Locus now has a focused first-touch benchmark that separates page materialization from arena reset allocation. On this local run, mapped first-touch is much slower than the default `Vec<u8>` baseline for 1 MiB, which is expected because the mapped path includes mapping and page fault work. This benchmark still does not prove NUMA placement.
