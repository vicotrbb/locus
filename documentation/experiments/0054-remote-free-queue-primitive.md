# Experiment 0054: Remote Free Queue Primitive

Date: 2026-07-02

## Postulate

See `documentation/postulates/0046-remote-free-queue-primitive.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed:

- `locus-alloc`: 23 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 17 unit tests passed.
- `locus-sys`: 5 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

The new primitive includes:

- `RemoteFreeQueue<T>` for owner-side draining;
- cloneable `RemoteFreeSink<T>` handles for remote producers;
- bounded queue capacity and drain batch limit;
- `drain_batch` with an owner-provided release closure;
- submitted and drained accounting;
- an enqueue error that returns the item if the owner is gone.

Focused tests cover batch draining, invalid configuration, and enqueue behavior after the owner is dropped.

## Conclusion

The postulate survived. Locus now has a safe owner-drained remote free queue primitive that can be benchmarked and later integrated with KV block or request arena release paths.

This is not yet a full allocator-specific remote-free batching design. It is the first reusable primitive for that work.
