# Experiment 0080: Placement Readiness Consistency

Date: 2026-07-02

## Postulate

See `documentation/postulates/0072-placement-readiness-consistency.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-observe
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed, including inconsistent placement readiness rejection.
- `locus-sys`: 5 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-observe` passed:

- `locus-observe`: 27 unit tests passed.
- Doc tests completed with no failures.

## Conclusion

The postulate survived. The placement readiness parser now rejects inconsistent status and reason pairs while continuing to accept valid ready and not-ready output.
