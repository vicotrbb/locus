# Experiment 0004: Linux Container Validation

Date: 2026-07-02

## Purpose

Validate the current workspace inside a Linux container before introducing Linux memory-policy syscalls.

## Commands

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test --workspace
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc 'rustup component add clippy && cargo clippy --workspace --all-targets -- -D warnings'
```

## Results

`cargo test --workspace` passed inside the Linux container:

- `locus-alloc`: 4 unit tests passed.
- `locus-core`: 5 unit tests passed.
- `locus-observe`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

The direct Linux Clippy command failed because `cargo-clippy` is not installed in the `rust:1.96` image used by Docker.

The attempted `rustup component add clippy` command also failed because this image does not include `rustup`.

## Conclusion

The current code and tests are Linux-compatible in the official Rust container. Linux Clippy remains unvalidated until the project uses a container image that includes the Clippy component or installs a toolchain with `rustup`.

## Next Questions

- Should Locus add a small development container image with Rust, Clippy, and Linux observability tools installed?
- Should the first Linux-only allocator experiment run in that image with `/proc/self/numa_maps` snapshots captured before and after arena allocation?
