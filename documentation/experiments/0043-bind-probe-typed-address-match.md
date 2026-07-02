# Experiment 0043: Bind Probe Typed Address Match

Date: 2026-07-02

## Postulate

See `documentation/postulates/0035-bind-probe-typed-address-match.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

Output:

```text
mapping_start=0xffff8ec8b000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
```

## Conclusion

The postulate survived as a compile-time and failure-path check. The bind probe now uses the typed `numa_maps` address matcher, preserving unavailable-state behavior in the current Docker environment.
