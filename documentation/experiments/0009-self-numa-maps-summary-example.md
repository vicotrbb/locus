# Experiment 0009: Self Numa Maps Summary Example

Date: 2026-07-02

## Purpose

Keep the live `self_numa_maps` example aligned with the reusable `NumaMapsSummary` API.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example self_numa_maps
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 7 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed locally.

The Linux container example passed and printed:

```text
numa_maps=unavailable
```

## Conclusion

The live example now uses `NumaMapsSummary` and preserves the prior unavailable-state behavior in the Docker environment where `/proc/self/numa_maps` is absent.
