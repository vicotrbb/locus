# Experiment 0027: Locality Environment Probe

Date: 2026-07-02

## Postulate

See `documentation/postulates/0019-locality-environment-probe.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example locality_environment
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 10 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example locality_environment
```

Output:

```text
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
```

## Conclusion

The postulate survived. The combined example gives a compact readiness report for the current Linux observability surface and confirms that the present Docker environment still does not expose the evidence needed to prove page placement.
