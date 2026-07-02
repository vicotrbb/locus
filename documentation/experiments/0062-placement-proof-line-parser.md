# Experiment 0062: Placement Proof Line Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0054-placement-proof-line-parser.md`.

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
- `locus-observe`: 20 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker mapped scratch bind output:

```text
mapping_start=0xffffb6615000
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

The postulate survived. `locus-observe` now parses the final placement proof line into `NumaPlacementProof` and rejects malformed, duplicate, extra, and unknown tokens with typed errors.

Docker still does not prove NUMA placement in this environment. It continues to validate the unavailable-evidence path and provides a final proof line that the new parser accepts.
