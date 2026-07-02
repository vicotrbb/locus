# Experiment 0015: Write-Touch Mapped Pages

Date: 2026-07-02

## Postulate

See `documentation/postulates/0012-write-touch-mapped-pages.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 18 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Linux container validation passed:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

The container run reported 5 `locus-sys` unit tests passed and doc tests completed with no failures.

## Conclusion

The postulate survived. Locus now has a safe page-size query and write-touch helper for owned mapped regions, validated locally and in the Linux Rust container.

This prepares mapped arena page materialization for future NUMA policy and page-placement experiments. It does not yet apply a NUMA policy or prove placement.
