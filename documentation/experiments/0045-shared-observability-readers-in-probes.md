# Experiment 0045: Shared Observability Readers In Probes

Date: 2026-07-02

## Postulate

See `documentation/postulates/0037-shared-observability-readers-in-probes.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example locality_environment
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example cgroup_numa_stat
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
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

Docker locality environment output:

```text
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
```

Docker cgroup NUMA stat output:

```text
cgroup_numa_stat=unavailable
```

Docker mapped scratch bind output:

```text
mapping_start=0xffff968b0000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
```

## Conclusion

The postulate survived. The live probes now reuse shared cgroup summary and node system snapshot readers while preserving unavailable-state behavior in Docker.
