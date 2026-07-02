# Experiment 0064: Placement Readiness Line Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0056-placement-readiness-line-parser.md`.

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
- `locus-observe`: 23 unit tests passed.
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

The postulate survived. `locus-observe` now parses the final placement validation readiness line into `NumaPlacementValidationReadiness` and rejects malformed, duplicate, extra, and unknown tokens with typed errors.

Docker still reports a not-ready placement validation environment because primary `numa_maps` evidence is unavailable.
