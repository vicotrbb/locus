# Experiment 0037: Node Numastat System Snapshot

Date: 2026-07-02

## Postulate

See `documentation/postulates/0029-node-numastat-system-snapshot.md`.

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
- `locus-observe`: 15 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. `locus-observe` now has a system-level node `numastat` snapshot and delta type for secondary locality evidence across visible NUMA nodes.
