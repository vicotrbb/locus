# Experiment 0070: Memory Policy Readiness Output Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0062-memory-policy-readiness-output-parser.md`.

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
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-sys` passed:

- `locus-sys`: 13 Linux unit tests passed, including valid and invalid memory-policy readiness output parsing.
- Doc tests completed with no failures.

Docker `mbind_region` output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=4
```

## Conclusion

The postulate survived. `locus-sys` now extracts Linux memory-policy readiness from multiline `mbind_region` output and rejects missing, duplicate, or malformed readiness lines.

Docker still denies `mbind`, so the output remains a not-ready permission-denied verdict.
