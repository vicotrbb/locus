# Experiment 0036: Cgroup Numa Delta Convenience

Date: 2026-07-02

## Postulate

See `documentation/postulates/0028-cgroup-numa-delta-convenience.md`.

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

The postulate survived. `CgroupNumaDelta` now exposes node lookup and non-zero detection helpers without changing signed delta semantics.
