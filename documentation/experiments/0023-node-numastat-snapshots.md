# Experiment 0023: Node Numastat Snapshots

Date: 2026-07-02

## Postulate

See `documentation/postulates/0017-node-numastat-snapshots.md`.

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
- `locus-observe`: 8 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. Locus now has reusable node `numastat` snapshots and signed deltas for before-after locality counter experiments.

This still depends on a Linux environment exposing node `numastat` files for live evidence.
