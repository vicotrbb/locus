# Experiment 0065: Linux Memory Policy Readiness

Date: 2026-07-02

## Postulate

See `documentation/postulates/0057-linux-memory-policy-readiness.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 23 unit tests passed.
- `locus-sys`: 5 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `mbind_region` output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=4
```

Docker `cargo test -p locus-sys` passed:

- `locus-sys`: 9 Linux unit tests passed, including the Linux NUMA policy readiness classifier.
- Doc tests completed with no failures.

## Conclusion

The postulate survived. `locus-sys` now exposes a typed Linux NUMA policy readiness verdict, and the `mbind_region` example prints a final machine-readable readiness line.

Docker still denies `mbind`, but that denial is now classified as `memory_policy_readiness=not_ready reason=permission_denied`.
