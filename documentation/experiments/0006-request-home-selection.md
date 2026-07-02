# Experiment 0006: Request Home Selection

Date: 2026-07-02

## Postulate

See `documentation/postulates/0004-request-home-selection.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 4 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. Locus now has deterministic request-home selection that honors scheduler-provided node affinity, worker CPU topology, and no-topology fallback behavior without introducing system calls.

## Next Questions

- Should GPU BDF locality lower `GpuId` into a preferred node before request admission?
- Should request home selection account for node pressure before the first arena manager experiment?
