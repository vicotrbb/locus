# Experiment 0073: Live Placement Validation Gate

Date: 2026-07-02

## Postulate

See `documentation/postulates/0065-live-placement-validation-gate.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
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

Docker live gate output:

```text
mapping_start=0xffff8c1cd000
mapping_len=20479
memory_policy_readiness=not_ready reason=permission_denied
touched=5
home_node=0
placement_validation_readiness=not_ready reason=numa_maps_unavailable
placement_proof=unavailable reason=numa_maps_unavailable
placement_validation_gate=not_ready reason=memory_policy_not_ready
```

Docker `cargo test -p locus-validate` passed:

- `locus-validate`: 4 Linux unit tests passed.
- Doc tests completed with no failures.

## Conclusion

The postulate survived. `locus-validate` now has a one-command live placement validation gate that attempts the mapped scratch workflow and prints the combined gate.

Docker still cannot prove placement. The live gate correctly reports `not_ready reason=memory_policy_not_ready` because `mbind` is denied before placement evidence can prove the mapping.
