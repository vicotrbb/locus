# Experiment 0032: Numa Placement Evidence Classifier

Date: 2026-07-02

## Postulate

See `documentation/postulates/0024-numa-placement-evidence-classifier.md`.

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
- `locus-observe`: 13 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. `locus-observe` now provides a tested placement-evidence classifier that only reports all-expected placement when every reported page in a matched `numa_maps` entry is on the expected node.
