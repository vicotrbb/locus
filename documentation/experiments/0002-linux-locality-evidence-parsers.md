# Experiment 0002: Linux Locality Evidence Parsers

Date: 2026-07-02

## Postulate

See `documentation/postulates/0002-linux-locality-evidence-parsers.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 4 unit tests passed.
- `locus-core`: 5 unit tests passed.
- `locus-observe`: 4 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived this parser-only step. Locus now has a safe parser layer for key Linux locality evidence formats, which can support later allocator validation without requiring Linux syscalls in the parser tests.

## Next Questions

- Should the next validation layer read live Linux files behind `cfg(target_os = "linux")`?
- Should allocator experiments record raw locality evidence as files under `documentation/experiments/raw/`?
