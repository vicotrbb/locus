# Experiment 0003: Observability File Readers

Date: 2026-07-02

## Postulate

See `documentation/postulates/0003-observability-file-readers.md`.

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
- `locus-observe`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. Locus now has tested file readers for explicit locality evidence paths plus a convenience `/proc/self/numa_maps` reader. The implementation remains safe and fixture-testable on non-Linux hosts.

## Next Questions

- Should the next allocator validation run inside an OrbStack Linux container?
- Should live evidence snapshots be stored as raw experiment artifacts or summarized only in Markdown?
