# Experiment 0084: Seccomp Unconfined Validation

Date: 2026-07-02

## Postulate

See `documentation/postulates/0076-seccomp-unconfined-validation.md`.

## Commands

```sh
docker run --rm --security-opt seccomp=unconfined -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
docker run --rm --security-opt seccomp=unconfined -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_placement_validation_gate
```

## Results

Executed on 2026-07-02.

Docker `cargo run -p locus-sys --example mbind_region` with seccomp disabled:

```text
mbind=error mbind syscall failed: Function not implemented (os error 38)
memory_policy_readiness=not_ready reason=syscall_failed
seccomp=disabled seccomp_filters=0 no_new_privs=0
touched=4
```

Docker `cargo run -p locus-validate --example live_placement_validation_gate` with seccomp disabled:

```text
mapping_start=0xffffa7c08000
mapping_len=20479
memory_policy_readiness=not_ready reason=syscall_failed
seccomp=disabled seccomp_filters=0 no_new_privs=0
touched=5
home_node=0
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
placement_proof=unavailable reason=numa_maps_unavailable
placement_validation_gate=not_ready reason=memory_policy_not_ready
```

## Conclusion

The postulate failed. Disabling seccomp changed the diagnostic from `seccomp=filter` to `seccomp=disabled`, but `mbind` still did not become ready. The failure changed from `Operation not permitted` to `Function not implemented`.

The current Docker environment still cannot prove NUMA placement. The next validation environment must provide an implemented and permitted `mbind` syscall, not only a disabled seccomp filter.
