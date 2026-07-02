# Experiment 0069: Placement Readiness Output Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0061-placement-readiness-output-parser.md`.

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
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 5 host-visible unit tests passed.
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

The postulate survived. `locus-observe` now extracts placement validation readiness from multiline locality environment output and rejects missing, duplicate, or malformed readiness lines.

Docker still reports a not-ready environment because primary `numa_maps` evidence is unavailable.
