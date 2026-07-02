# Experiment 0044: Cgroup Numa Summary Reader

Date: 2026-07-02

## Postulate

See `documentation/postulates/0036-cgroup-numa-summary-reader.md`.

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
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. `locus-observe` now exposes a shared explicit-path cgroup NUMA summary reader that reuses the existing parser and read error model.
