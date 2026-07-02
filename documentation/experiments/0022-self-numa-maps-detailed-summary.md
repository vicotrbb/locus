# Experiment 0022: Self Numa Maps Detailed Summary

Date: 2026-07-02

## Purpose

Update the live `self_numa_maps` example to print policy and kernel page-size summaries alongside node page totals.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example self_numa_maps
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example self_numa_maps
```

Output:

```text
numa_maps=unavailable
```

## Conclusion

The example now prints policy and kernel page-size detail when `numa_maps` is available. The current Docker environment still does not expose `/proc/self/numa_maps`, and that unavailable state is handled explicitly.
