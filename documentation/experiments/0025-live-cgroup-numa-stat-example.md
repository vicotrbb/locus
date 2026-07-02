# Experiment 0025: Live Cgroup Numa Stat Example

Date: 2026-07-02

## Purpose

Add a live cgroup v2 `memory.numa_stat` example so Locus can inspect per-node cgroup memory counters when the current environment exposes them.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example cgroup_numa_stat
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 19 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 8 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example cgroup_numa_stat
```

Output:

```text
cgroup_numa_stat=unavailable
```

## Conclusion

The cgroup `memory.numa_stat` example is runnable and reports unavailable state cleanly. The current Docker environment does not expose cgroup NUMA memory counters, so live cgroup placement evidence remains unavailable here.
