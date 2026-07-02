# Experiment 0071: Combined Placement Validation Gate

Date: 2026-07-02

## Postulate

See `documentation/postulates/0063-combined-placement-validation-gate.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example locality_environment
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 5 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-validate` passed:

- `locus-validate`: 4 Linux unit tests passed.
- Doc tests completed with no failures.

Docker `mbind_region` output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=4
```

Docker locality environment output:

```text
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
```

Docker mapped scratch bind output:

```text
mapping_start=0xffff860ed000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
placement_proof=unavailable reason=numa_maps_unavailable
```

## Conclusion

The postulate survived. `locus-validate` now provides a Linux-gated combined placement validation gate that parses the three probe outputs and requires ready memory policy, ready evidence, and verified placement proof before returning a verified gate.

The current Docker outputs evaluate as `not_ready` with reason `memory_policy_not_ready`, which is correct because `mbind` is denied before placement evidence can prove anything.
