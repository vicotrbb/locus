# Experiment 0005: Live Self Numa Maps Example

Date: 2026-07-02

## Purpose

Validate that `locus-observe` can read live `/proc/self/numa_maps` data in a Linux container and summarize page counts by NUMA node.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example self_numa_maps
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 4 unit tests passed.
- `locus-core`: 5 unit tests passed.
- `locus-observe`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed locally.

The first Linux container run failed because `/proc/self/numa_maps` was not present in the container environment:

```text
Error: Read { path: "/proc/self/numa_maps", source: Os { code: 2, kind: NotFound, message: "No such file or directory" } }
```

The example was updated to treat a missing `numa_maps` file as an explicit unavailable state. The final Linux container run passed and printed:

```text
numa_maps=unavailable
```

## Conclusion

The example is usable in Linux environments that expose `/proc/self/numa_maps` and degrades cleanly when the proc file is absent. This container cannot validate live NUMA page placement, but it does validate the reader path and unavailable-state handling.

## Next Questions

- Should the next scratch arena experiment write-touch a larger arena before collecting `numa_maps`?
- Should page-size and mapping-policy summaries be added to the example output?
