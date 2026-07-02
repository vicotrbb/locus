# Experiment 0029: Mapped Arena Placement Identity

Date: 2026-07-02

## Postulate

See `documentation/postulates/0021-mapped-arena-placement-identity.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 20 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 11 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker command:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

Output:

```text
mapping_start=0xffff8ceb0000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
touched=5
home_node=0
```

## Conclusion

The postulate survived. Mapped scratch arenas now expose safe mapping identity, parsed `numa_maps` entries can be matched by exact start address, and the live bind probe prints the address range needed for future placement evidence. The current Docker environment still prevents a successful NUMA policy application.
