# Experiment 0038: Live Node Numastat System Reader

Date: 2026-07-02

## Postulate

See `documentation/postulates/0030-live-node-numastat-system-reader.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example node_numastat
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 16 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example node_numastat
```

Output:

```text
node_numastat=unavailable
```

## Conclusion

The postulate survived. `locus-observe` now has a shared live reader for system-level node `numastat` snapshots, and the live example uses that reader while preserving unavailable-state behavior in Docker.
