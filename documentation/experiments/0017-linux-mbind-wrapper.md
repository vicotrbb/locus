# Experiment 0017: Linux Mbind Wrapper

Date: 2026-07-02

## Postulate

See `documentation/postulates/0014-linux-mbind-wrapper.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 5 unit tests passed on the local host build.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Linux container validation passed:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

The container run reported 8 `locus-sys` unit tests passed, including Linux-only node-mask tests, and doc tests completed with no failures.

## Conclusion

The postulate survived as a syscall-wrapper foundation. `locus-sys` now exposes a Linux-only `bind_region_to_node` helper and validates node-mask construction in the Linux container.

This does not yet prove that `mbind` succeeds in the target runtime or that pages land on the requested NUMA node. The next step should be an opt-in Linux integration experiment that applies the policy in a container or NUMA-capable host and records `numa_maps` evidence.
