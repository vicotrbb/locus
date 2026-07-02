# Experiment 0034: Cgroup Numa Summary Delta

Date: 2026-07-02

## Postulate

See `documentation/postulates/0026-cgroup-numa-summary-delta.md`.

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
- `locus-observe`: 14 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. `locus-observe` now computes signed aggregate and per-node cgroup NUMA deltas, including nodes that disappear or newly appear between snapshots. This remains secondary evidence rather than proof of page placement.
