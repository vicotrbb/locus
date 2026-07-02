# Experiment 0019: Mapped Scratch Linux Bind

Date: 2026-07-02

## Postulate

See `documentation/postulates/0015-mapped-scratch-linux-bind.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
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

Linux container validation passed:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

The container run reported 20 `locus-alloc` unit tests passed, including the Linux-only invalid bind-node test, and doc tests completed with no failures.

## Conclusion

The postulate survived. `MappedScratchArena` now exposes a Linux-only `bind_to_node` method that delegates to the `locus-sys` policy boundary and wraps policy errors in the allocator error type.

This still does not prove successful placement. The test validates safe error handling for invalid node masks. A permitted Linux environment is still needed to validate a successful bind and resulting `numa_maps` placement.
