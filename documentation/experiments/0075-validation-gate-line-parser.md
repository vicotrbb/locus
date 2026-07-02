# Experiment 0075: Validation Gate Line Parser

Date: 2026-07-02

## Postulate

See `documentation/postulates/0067-validation-gate-line-parser.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_placement_validation_gate
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

- `locus-validate`: 8 Linux unit tests passed, including valid and invalid placement validation gate line and output parsing.
- Doc tests completed with no failures.

Docker live gate output:

```text
mapping_start=0xffffbc723000
mapping_len=20479
memory_policy_readiness=not_ready reason=permission_denied
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

The postulate survived. `locus-validate` now parses the final combined placement validation gate line and extracts the gate verdict from multiline live output.

Docker still reports a not-ready gate because memory policy application is denied.
