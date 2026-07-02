# Experiment 0016: Mapped Scratch Page Touch

Date: 2026-07-02

## Postulate

See `documentation/postulates/0013-mapped-scratch-page-touch.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. `MappedScratchArena` now exposes safe page materialization through the same allocator object that future locality experiments will inspect.

This still does not validate NUMA placement. It makes the mapped scratch arena ready for policy and observability experiments.
