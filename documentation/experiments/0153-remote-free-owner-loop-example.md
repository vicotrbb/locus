# Experiment 0153: Remote-Free Owner Loop Example

Date: 2026-07-03

## Postulate

[Postulate 0145](../postulates/0145-remote-free-owner-loop-example.md) claimed that `RemoteFreeDrainController` needs a runtime-facing owner-loop example that is compile-checked and tested with real allocated buffers, not only benchmark-local call sites.

## Change

Added rustdoc guidance to `RemoteFreeDrainController` showing the intended owner-loop structure:

- create a `RemoteFreeQueue`;
- create a `RemoteFreeDrainController` with a policy;
- enqueue remotely released domain items through a sink;
- record submitted queued bytes;
- ask the controller whether the owner should drain;
- drain the queue with explicit domain release logic;
- record drained bytes with the controller.

Added `remote_free_drain_controller_owner_loop_releases_allocated_buffers`, a focused unit test that exercises the same pattern with real `Vec` buffers.

## Validation

Host commands:

```bash
cargo test -p locus-alloc remote_free_drain_controller_owner_loop_releases_allocated_buffers
cargo test -p locus-alloc --doc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Validation results:

- `cargo test -p locus-alloc remote_free_drain_controller_owner_loop_releases_allocated_buffers`: passed, 1 focused test.
- `cargo test -p locus-alloc --doc`: passed, 1 doc test.
- `cargo test --workspace`: passed, 166 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

## Interpretation

The postulate survived.

The rustdoc example is compile-checked by doc tests, and the focused unit test exercises the same owner-loop pattern with actual heap-backed `Vec` buffers passing through `RemoteFreeQueue`. The controller still does not hide release logic: the closure computes released bytes from each buffer and records the drain explicitly.

This step is not a throughput benchmark. It converts the measured benchmark pattern into a small runtime-facing template with test coverage.

## Next Step

Consider adding a typed owner-loop adapter only if repeated runtime call sites need it. Until then, keep release closures explicit so KV, request scratch, pinned staging, and mixed buffers can report domain-specific release bytes correctly.
