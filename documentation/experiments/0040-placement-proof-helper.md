# Experiment 0040: Placement Proof Helper

Date: 2026-07-02

## Postulate

See `documentation/postulates/0032-placement-proof-helper.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 16 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. `NumaPlacementEvidence` now exposes explicit helpers for the conservative placement proof condition and for counting pages observed on other nodes.
