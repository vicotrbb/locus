# Experiment 0018: Linux Mbind Probe Example

Date: 2026-07-02

## Purpose

Add a runnable Linux example that invokes the `mbind` wrapper on an owned mapped region and records whether the current container supports the syscall.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
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

Docker probe command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
```

Output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
touched=4
```

## Conclusion

The probe successfully invoked the Linux `mbind` wrapper in Docker and then write-touched the mapping. This container rejects the policy call with `EPERM`, so it cannot prove successful placement. The result is still useful: the syscall path is runnable, errors are surfaced cleanly, and page materialization proceeds after the failed policy attempt.

The next placement validation needs a container or host configuration that permits `mbind` and exposes `/proc/self/numa_maps`.
