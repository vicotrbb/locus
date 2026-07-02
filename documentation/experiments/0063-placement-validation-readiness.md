# Experiment 0063: Placement Validation Readiness

Date: 2026-07-02

## Postulate

See `documentation/postulates/0055-placement-validation-readiness.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example locality_environment
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 21 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker locality environment output:

```text
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
```

## Conclusion

The postulate survived. `locus-observe` now has a typed placement validation readiness helper, and the locality environment example prints a final machine-readable readiness verdict.

The current Docker environment remains not ready for successful placement validation because primary `numa_maps` evidence is unavailable.
