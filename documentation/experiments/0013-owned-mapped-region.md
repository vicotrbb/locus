# Experiment 0013: Owned Mapped Region

Date: 2026-07-02

## Postulate

See `documentation/postulates/0010-owned-mapped-region.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 15 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 2 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Linux container validation passed:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

The container run reported 2 `locus-sys` unit tests passed and doc tests completed with no failures.

## Conclusion

The postulate survived. Locus now has a narrow unsafe system boundary with a safe owned anonymous mapped region, validated locally and inside the Linux Rust container.

This does not yet apply NUMA policy or inspect physical page placement. It provides the owned address range needed for those next steps.
