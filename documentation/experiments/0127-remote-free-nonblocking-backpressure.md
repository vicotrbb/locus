# Experiment 0127: Remote Free Nonblocking Backpressure

Date: 2026-07-03

## Postulate

[Postulate 0119](../postulates/0119-remote-free-nonblocking-backpressure.md) claimed that `RemoteFreeSink` can expose a nonblocking enqueue path with explicit queue-full backpressure while preserving the existing blocking enqueue behavior.

## Change

Added `RemoteFreeSink::try_enqueue` to `locus-alloc`.

The new API:

- returns the item on full-queue and disconnected-owner failures;
- distinguishes failure kind with `RemoteFreeTryEnqueueErrorKind`;
- counts successful submissions, pending items, full-queue rejections, disconnected attempts, and owner-drained items;
- exposes the counters through `RemoteFreeQueueStats`;
- keeps `RemoteFreeSink::enqueue` as the existing blocking path;
- increments disconnected accounting when either enqueue path observes a dropped owner.

Focused tests now cover full-queue backpressure, item return, disconnected-owner accounting, and the expanded stats surface.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
cargo fmt --all -- --check
git diff --check
rg -n "<literal em dash>" documentation crates README.md Cargo.toml Cargo.lock || true
```

## Results

- Host `cargo test -p locus-alloc`: 59 unit tests passed, plus doc tests.
- Host `cargo test --workspace`: all workspace tests passed.
- Host `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker `docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc`: 61 unit tests passed, plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Em dash scan: no matches.

## Conclusion

The postulate survived. `RemoteFreeQueue` now has a scheduler-visible congestion signal without changing the existing blocking enqueue path or current benchmark call sites.

This change was validated with unit tests rather than a new timing benchmark because it adds control-plane behavior and accounting. The next benchmark-facing step should use `try_enqueue` in a mixed remote-return workload to measure how often bounded queues saturate under different drain policies.
