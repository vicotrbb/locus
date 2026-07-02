# Experiment 0060: Placement Proof Verdict

Date: 2026-07-02

## Postulate

See `documentation/postulates/0052-placement-proof-verdict.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 18 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker mapped scratch bind output:

```text
mapping_start=0xffffa3e42000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
placement_proof=unavailable reason=numa_maps_unavailable
```

## Conclusion

The postulate survived. The observe crate now has a reusable primary placement proof verdict helper, and the mapped scratch bind probe emits a final `placement_proof` line.

The Docker run still does not prove NUMA placement. It confirms failure and unavailable-evidence reporting. A successful proof still requires an environment where `mbind` is permitted and `/proc/self/numa_maps` is readable.
