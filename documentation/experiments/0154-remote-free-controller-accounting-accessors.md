# Experiment 0154: Remote-Free Controller Accounting Accessors

Date: 2026-07-03

## Postulate

[Postulate 0146](../postulates/0146-remote-free-controller-accounting-accessors.md) claimed that `RemoteFreeDrainController` should expose direct owner-side accounting accessors so runtime code and benchmarks do not need to reach through `controller.tracker()` for common pending count, queued byte, and empty-state checks.

## Change

Added direct controller accessors:

- `RemoteFreeDrainController::pending_count`;
- `RemoteFreeDrainController::queued_bytes`;
- `RemoteFreeDrainController::is_empty`.

Updated:

- the rustdoc owner-loop example;
- focused controller tests;
- `request_remote_free_policy`;
- `kv_remote_free_policy`;
- `remote_free_mixed_size_policy`.

`RemoteFreeDrainController::tracker` remains available for explicit low-level inspection.

## Validation

Host commands:

```bash
cargo bench -p locus-alloc --bench request_remote_free_policy --no-run
cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
cargo test -p locus-alloc remote_free_drain_controller
cargo test -p locus-alloc --doc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Validation results:

- `request_remote_free_policy` no-run compile: passed.
- `kv_remote_free_policy` no-run compile: passed.
- `remote_free_mixed_size_policy` no-run compile: passed.
- `cargo test -p locus-alloc remote_free_drain_controller`: passed, 4 focused tests.
- `cargo test -p locus-alloc --doc`: passed, 1 doc test.
- `cargo test --workspace`: passed, 166 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Interpretation

The postulate survived.

The direct accessors report the controller's tracked pending count, queued bytes, and empty state without exposing benchmark code to `RemoteFreeDrainTracker` internals. The policy benchmarks still compile with the same controller-driven control flow, and the focused controller status test now asserts the direct accessor values.

This is not a throughput benchmark. It is an API clarity step for runtime owner loops after the measured controller migrations in experiments 0150 through 0153.

## Next Step

Move back to measurement work, preferably a placement or page-touch validation step, so allocator-locality claims continue to be tested against Linux evidence rather than API shape alone.
