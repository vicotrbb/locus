# Experiment 0082: Memory Readiness Display

Date: 2026-07-02

## Postulate

See `documentation/postulates/0074-memory-readiness-display.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 5 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-sys` passed:

- `locus-sys`: 15 Linux unit tests passed, including memory readiness display coverage.
- Doc tests completed with no failures.

Docker `cargo run -p locus-sys --example mbind_region` output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
seccomp=filter seccomp_filters=1 no_new_privs=0
touched=4
```

## Conclusion

The postulate survived. Memory-policy readiness probes now use the verdict display representation while preserving the stable final machine-readable line.
