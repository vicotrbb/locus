# Experiment 0078: Linux Seccomp Diagnostic

Date: 2026-07-02

## Postulate

See `documentation/postulates/0070-linux-seccomp-diagnostic.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_placement_validation_gate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
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

Docker `cargo test -p locus-sys` passed:

- `locus-sys`: 15 Linux unit tests passed, including seccomp diagnostic parsing and formatting.
- Doc tests completed with no failures.

Docker `cargo run -p locus-sys --example mbind_region` output:

```text
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
seccomp=filter seccomp_filters=1 no_new_privs=0
touched=4
```

Docker `cargo run -p locus-validate --example live_placement_validation_gate` output:

```text
mapping_start=0xffff8ecbf000
mapping_len=20479
memory_policy_readiness=not_ready reason=permission_denied
seccomp=filter seccomp_filters=1 no_new_privs=0
touched=5
home_node=0
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
placement_proof=unavailable reason=numa_maps_unavailable
placement_validation_gate=not_ready reason=memory_policy_not_ready
```

Docker `cargo test -p locus-validate` passed:

- `locus-validate`: 8 Linux unit tests passed.
- Doc tests completed with no failures.

## Conclusion

The postulate survived. The Docker validation environment reports `seccomp=filter seccomp_filters=1`, which is relevant context for the `mbind` permission denial.

This does not prove seccomp is the only reason for `EPERM`, but it makes the current placement-proof blocker more diagnosable.
