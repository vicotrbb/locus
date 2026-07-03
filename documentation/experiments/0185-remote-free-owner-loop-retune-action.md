# Experiment 0185: Remote-Free Owner-Loop Retune Action

Date: 2026-07-03

## Postulate

[Postulate 0177](../postulates/0177-remote-free-owner-loop-retune-action.md)
claimed that the queued-byte owner-loop example should log the same
`RemoteFreeQueuedByteRetuneAction` recommendation that the retune benchmarks
assert, without changing the owner-side release path or measured queue
counters.

## Change

Updated `remote_free_queued_byte_owner_loop` to:

- call `RemoteFreeDrainController::status_for_queue` at each owner control
  point;
- build `RemoteFreeQueuedByteDriftReport` from that status;
- track max pending over-target, max queued bytes over-budget, queue
  backpressure observation, `retune_hint`, and `retune_action`;
- print those fields in the stable completion line.

The example still releases real `Vec<u8>` allocations in the
`RemoteFreeQueue::drain_batch` closure.

## Validation

Commands:

```bash
cargo fmt --all
cargo run -q -p locus-alloc --example remote_free_queued_byte_owner_loop
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- Owner-loop example passed.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 226 unit tests and 3 `locus_alloc` doctests passed.

## Owner-Loop Output

```text
remote_free_queued_byte_owner_loop=started
queue_capacity=256
drain_batch_limit=64
request_concurrency=4
remote_free_blocks_per_request=16
representative_block_bytes=10240
target_pending_items=64
queued_byte_budget=655360
remote_free_queued_byte_owner_loop=complete blocks=256 submitted_count=256 drained_count=256 pending_count=0 full_count=0 forced_drains=0 policy_drains=4 drain_rounds=4 max_pending_count=64 max_queued_bytes=655360 released_bytes=2621440 max_wait_bursts=2 mean_wait_bursts=1.500 max_pending_over_target=0 max_queued_bytes_over_budget=0 queue_backpressure_observed=0 retune_hint=keep_config retune_action=keep_config
```

## Interpretation

The postulate survived.

Adding drift observation to the runtime-facing owner-loop example preserved the
existing measured counters:

- `full_count=0`;
- `policy_drains=4`;
- max pending 64;
- max queued bytes 655,360;
- max wait 2 bursts;
- mean wait 1.500 bursts.

The final drift report stayed clean: no pending over-target, no queued-byte
over-budget, no queue backpressure, `retune_hint=keep_config`, and
`retune_action=keep_config`.

This validates a service-facing pattern: collect a controller status at the
owner control point, derive a drift report, log the non-mutating retune action,
then drain according to the configured policy.

## Next Step

Move the same drift-action logging pattern into one domain-specific owner loop,
starting with KV block handles or request-affine arenas.
