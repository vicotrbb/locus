# Experiment 0026: Current Cgroup Numa Stat Path

Date: 2026-07-02

## Postulate

See `documentation/postulates/0018-current-cgroup-numa-stat-path.md`.

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
- `locus-observe`: 10 unit tests passed.
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

The postulate survived. Locus now resolves the current cgroup v2 memory NUMA stat path from `/proc/self/cgroup`, and the live example still handles unavailable cgroup NUMA counters explicitly in Docker.
