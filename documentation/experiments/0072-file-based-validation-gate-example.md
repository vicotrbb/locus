# Experiment 0072: File-Based Validation Gate Example

Date: 2026-07-02

## Postulate

See `documentation/postulates/0064-file-based-validation-gate-example.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -v "$tmpdir":/captures -w /work rust:1.96 cargo run -p locus-validate --example placement_validation_gate -- /captures/memory.txt /captures/readiness.txt /captures/proof.txt
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

The Docker example consumed three captured output files equivalent to the current Docker probe verdicts and printed:

```text
placement_validation_gate=not_ready reason=memory_policy_not_ready
```

## Conclusion

The postulate survived. `locus-validate` now has a runnable Linux example that consumes captured probe outputs and prints a single combined placement validation gate verdict.

Current Docker-like captures correctly evaluate as not ready because memory policy application is denied.
