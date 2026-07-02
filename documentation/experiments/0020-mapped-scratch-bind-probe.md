# Experiment 0020: Mapped Scratch Bind Probe

Date: 2026-07-02

## Purpose

Run the Linux bind probe through the allocator layer by applying `MappedScratchArena::bind_to_node`, write-touching pages, and recording the current container behavior.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed on the local host build.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker probe command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

Output:

```text
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
touched=5
home_node=0
```

## Conclusion

The allocator-level probe ran successfully. It exercised `MappedScratchArena::bind_to_node`, surfaced the container `EPERM` clearly, then materialized mapped pages through `MappedScratchArena::write_touch_pages`.

This still does not prove NUMA placement. It proves that allocator-level policy application is wired to the system boundary and that this Docker environment does not permit the bind operation.
