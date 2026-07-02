# Experiment 0066: Memory Policy Readiness Line Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0058-memory-policy-readiness-line-parser.md`.

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
- `locus-observe`: 23 unit tests passed.
- `locus-sys`: 5 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-sys` passed:

- `locus-sys`: 11 Linux unit tests passed, including valid and invalid memory-policy readiness line parsing.
- Doc tests completed with no failures.

Docker `mbind_region` output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=4
```

## Conclusion

The postulate survived. `locus-sys` now parses the final memory-policy readiness line into `LinuxNumaPolicyReadiness` and rejects malformed, duplicate, extra, and unknown tokens with typed errors.

Docker still denies `mbind`, and the probe output remains a not-ready permission-denied verdict.
