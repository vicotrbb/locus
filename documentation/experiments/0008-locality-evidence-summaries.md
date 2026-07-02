# Experiment 0008: Locality Evidence Summaries

Date: 2026-07-02

## Postulate

See `documentation/postulates/0006-locality-evidence-summaries.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 7 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. Locus now has shared summary types for `numa_maps` page totals and cgroup `memory.numa_stat` byte totals, with fixture coverage for per-node aggregation.

## Next Questions

- Should summaries include page size groups from `kernelpagesize_kB`?
- Should summaries report policy counts such as `default`, `bind`, and `interleave`?
