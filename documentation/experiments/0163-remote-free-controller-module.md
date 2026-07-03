# Experiment 0163: Remote-Free Controller Module

Date: 2026-07-03

## Postulate

[Postulate 0155](../postulates/0155-remote-free-controller-module.md)
claimed that the remote-free policy, tracker, and controller code should live in
a focused submodule instead of sharing one large `remote_free.rs` file with the
bounded handoff queue.

## Change

Extracted remote-free owner-side policy code into
`crates/locus-alloc/src/remote_free/controller.rs`.

The extracted module owns:

- `RemoteFreeDrainPolicy`;
- `RemoteFreeDrainObservation`;
- `RemoteFreeDrainDecision`;
- `RemoteFreeDrainReason`;
- `RemoteFreeDrainTracker`;
- `RemoteFreeTrackedDrain`;
- `RemoteFreeDrainController`;
- `RemoteFreeDrainControllerStatus`;
- tracker and controller error types.

`remote_free.rs` now owns the queue, sink, queue stats, queue errors, enqueue
errors, and queue tests. The public `locus_alloc::*` API remains stable through
the existing `remote_free` re-exports.

Line counts after extraction:

| File | Lines |
| --- | ---: |
| `crates/locus-alloc/src/remote_free.rs` before extraction | 1211 |
| `crates/locus-alloc/src/remote_free.rs` after extraction | 440 |
| `crates/locus-alloc/src/remote_free/controller.rs` after extraction | 788 |

## Validation

Host commands:

```bash
cargo test -p locus-alloc remote_free
cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run
cargo bench -p locus-alloc --bench request_remote_free_policy --no-run
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Results:

- `cargo test -p locus-alloc remote_free`: passed, 17 focused tests.
- `cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run`: passed.
- `cargo bench -p locus-alloc --bench request_remote_free_policy --no-run`:
  passed.
- `cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run`:
  passed.
- `cargo test --workspace`: passed, 191 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Short real mixed-size benchmark output:

```text
remote_free_mixed_size_policy_sample=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=256 drained_count=256 full_count=0 forced_drains=0 policy_drains=0 drain_rounds=4 max_pending_count=256 max_queued_bytes=2621440 released_bytes=2621440 max_wait_bursts=8 mean_wait_bursts=4.500
remote_free_mixed_size_trace_capacity256_batch64_end_drain
                        time:   [41.754 us 41.839 us 41.900 us]
remote_free_mixed_size_policy_sample=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 submitted_count=256 drained_count=256 full_count=0 forced_drains=0 policy_drains=4 drain_rounds=4 max_pending_count=64 max_queued_bytes=655360 released_bytes=2621440 max_wait_bursts=2 mean_wait_bursts=1.500
remote_free_mixed_size_trace_capacity256_batch64_max_wait2
                        time:   [37.785 us 38.117 us 38.437 us]
```

## Interpretation

The postulate survived.

The extraction preserved queue behavior, controller behavior, public API imports,
doc tests, and benchmark compilation. The short mixed-size benchmark exercised
the real remote-free allocation path through `RemoteFreeQueue` and the extracted
`RemoteFreeDrainController`. Counters preserved the expected policy behavior:
end-drain reached max pending 256 and max-wait-2 reduced max pending to 64,
reduced peak queued bytes to 655,360, and kept `full_count=0`.

This is not a new best benchmark result. It is architecture evidence that the
measured controller behavior can live behind a cleaner module boundary without
changing runtime-facing imports or release-closure ownership.

## Next Step

Keep future queue primitive changes in `remote_free.rs` and future policy,
tracker, and owner-loop changes in `remote_free/controller.rs`. If either module
continues to grow, split by behavior only after another measured call path needs
the added code.
